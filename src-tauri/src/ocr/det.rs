//! PP-OCRv5 mobile detection: preprocess -> ONNX inference -> DB post-process.

use std::error::Error;
use std::path::PathBuf;
use std::time::Instant;

use image::{GrayImage, Rgba, RgbaImage, imageops::FilterType};
use imageproc::contours::{BorderType, find_contours};
use imageproc::drawing::draw_line_segment_mut;
use imageproc::geometry::min_area_rect;
use ort::value::Tensor;

use crate::ocr::Pt;

use super::OcrState;

// ---- Settings (from inference.yml) ----------------------------------------
const RESIZE_LONG: f32 = 960.0;
const THRESH: f32 = 0.3;
const BOX_THRESH: f32 = 0.6;
const MAX_CANDIDATES: usize = 1000;
const UNCLIP_RATIO: f32 = 1.5;
const MIN_SIZE: f32 = 3.0;

// ---- Debug switch ----------------------------------------------------------
const DEBUG: bool = false;

macro_rules! dlog {
    ($($arg:tt)*) => {
        if DEBUG {
            println!("[det] {}", format!($($arg)*));
        }
    };
}

/// A detected text box in ORIGINAL image coordinates, ordered tl, tr, br, bl.
#[derive(Debug, Clone)]
pub struct TextBox {
    pub pts: [Pt; 4],
    pub score: f32,
}

struct DetInput {
    data: Vec<f32>, // planar CHW, BGR order
    w: u32,
    h: u32,
    scale_x: f32, // resized / original
    scale_y: f32,
}

// ============================================================================
// Preprocess
// ============================================================================

fn preprocess_det(img: &RgbaImage) -> DetInput {
    let (w, h) = img.dimensions();

    let ratio = RESIZE_LONG / w.max(h) as f32;
    let round32 = |v: f32| (((v / 32.0).round() as u32).max(1)) * 32;
    let new_w = round32(w as f32 * ratio);
    let new_h = round32(h as f32 * ratio);

    let resized = image::imageops::resize(img, new_w, new_h, FilterType::Triangle);

    let mean = [0.485_f32, 0.456, 0.406];
    let std = [0.229_f32, 0.224, 0.225];

    let (nw, nh) = (new_w as usize, new_h as usize);
    let plane = nw * nh;
    let mut data = vec![0.0_f32; 3 * plane];

    for (x, y, p) in resized.enumerate_pixels() {
        let bgr = [p[2], p[1], p[0]];
        let idx = y as usize * nw + x as usize;
        for c in 0..3 {
            data[c * plane + idx] = (bgr[c] as f32 / 255.0 - mean[c]) / std[c];
        }
    }

    DetInput {
        data,
        w: new_w,
        h: new_h,
        scale_x: new_w as f32 / w as f32,
        scale_y: new_h as f32 / h as f32,
    }
}

// ============================================================================
// Entry point
// ============================================================================

pub fn run_detection(state: &OcrState, img: &RgbaImage) -> Result<Vec<TextBox>, Box<dyn Error>> {
    let total = Instant::now();
    let (orig_w, orig_h) = img.dimensions();
    dlog!("input image: {}x{}", orig_w, orig_h);

    let debug_dir = if DEBUG { make_debug_dir() } else { None };

    // 1. Preprocess
    let t = Instant::now();
    let input = preprocess_det(img);
    dlog!(
        "preprocess: {}x{} (scale_x={:.3}, scale_y={:.3}) in {:?}",
        input.w,
        input.h,
        input.scale_x,
        input.scale_y,
        t.elapsed()
    );
    if DEBUG {
        let mn = input.data.iter().cloned().fold(f32::INFINITY, f32::min);
        let mx = input.data.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        dlog!(
            "tensor value range: {:.3} .. {:.3} (expect roughly -2.1 .. 2.6)",
            mn,
            mx
        );
    }

    let (scale_x, scale_y) = (input.scale_x, input.scale_y);
    let shape = [1_i64, 3, input.h as i64, input.w as i64];

    // 2. Inference
    let t = Instant::now();
    let (pred_w, pred_h, probs): (usize, usize, Vec<f32>) = {
        let mut model = state.detection.lock().map_err(|e| e.to_string())?;

        let outputs = model.run(ort::inputs!["x" => Tensor::from_array((shape, input.data))?])?;

        let (out_shape, data) = outputs[0].try_extract_tensor::<f32>()?;
        let dims: Vec<usize> = out_shape.iter().map(|&d| d as usize).collect();
        dlog!("output shape: {:?}, {} values", dims, data.len());

        if dims.len() < 2 {
            return Err(format!("unexpected output shape {:?}", dims).into());
        }
        let h = dims[dims.len() - 2];
        let w = dims[dims.len() - 1];
        if data.len() != h * w {
            return Err(format!("output has {} values but H*W = {}", data.len(), h * w).into());
        }
        (w, h, data.to_vec())
    };
    dlog!("inference: {:?}", t.elapsed());

    if DEBUG {
        let mn = probs.iter().cloned().fold(f32::INFINITY, f32::min);
        let mx = probs.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        let mean = probs.iter().sum::<f32>() / probs.len() as f32;
        let above = probs.iter().filter(|&&p| p > THRESH).count();
        dlog!(
            "prob map: min={:.3} max={:.3} mean={:.3}, {} px above {} ({:.1}%)",
            mn,
            mx,
            mean,
            above,
            THRESH,
            100.0 * above as f32 / probs.len() as f32
        );
        if mx < THRESH {
            dlog!(
                "WARNING: max prob is below the threshold, so nothing will be detected. Check preprocessing (BGR order, normalization)."
            );
        }
    }

    // 3. Post-process
    let t = Instant::now();
    let boxes = postprocess_det(
        &probs,
        pred_w,
        pred_h,
        scale_x,
        scale_y,
        orig_w,
        orig_h,
        debug_dir.as_ref(),
    );
    dlog!("postprocess: {} boxes in {:?}", boxes.len(), t.elapsed());

    for (i, b) in boxes.iter().enumerate() {
        dlog!(
            "  box {:>2}: score={:.3} tl=({:.0},{:.0}) br=({:.0},{:.0})",
            i,
            b.score,
            b.pts[0].0,
            b.pts[0].1,
            b.pts[2].0,
            b.pts[2].1
        );
    }

    // 4. Debug images
    if let Some(dir) = &debug_dir {
        save_debug_images(dir, img, &probs, pred_w, pred_h, &boxes);
        dlog!("debug images written to {}", dir.display());
    }

    dlog!("total: {:?}", total.elapsed());
    Ok(boxes)
}

// ============================================================================
// DB post-process
// ============================================================================

#[allow(clippy::too_many_arguments)]
fn postprocess_det(
    probs: &[f32],
    w: usize,
    h: usize,
    scale_x: f32,
    scale_y: f32,
    orig_w: u32,
    orig_h: u32,
    debug_dir: Option<&PathBuf>,
) -> Vec<TextBox> {
    // Binary mask
    let mask: Vec<u8> = probs
        .iter()
        .map(|&p| if p > THRESH { 255 } else { 0 })
        .collect();
    let bitmap = GrayImage::from_raw(w as u32, h as u32, mask).expect("mask size mismatch");
    if let Some(dir) = debug_dir {
        let _ = bitmap.save(dir.join("2_mask.png"));
    }

    let contours = find_contours::<i32>(&bitmap);
    let outer: Vec<_> = contours
        .iter()
        .filter(|c| c.border_type == BorderType::Outer)
        .collect();
    dlog!("contours: {} total, {} outer", contours.len(), outer.len());

    let (mut small, mut low_score, mut small_after) = (0, 0, 0);
    let mut boxes: Vec<TextBox> = Vec::new();

    for c in outer.into_iter().take(MAX_CANDIDATES) {
        if c.points.len() < 4 {
            small += 1;
            continue;
        }

        let rect = min_area_rect(&c.points);
        let pts = order_points(rect.map(|p| (p.x as f32, p.y as f32)));

        let (sw, sh) = side_lengths(&pts);
        if sw.min(sh) < MIN_SIZE {
            small += 1;
            continue;
        }

        let score = box_score(probs, w, h, &pts);
        if score < BOX_THRESH {
            low_score += 1;
            continue;
        }

        let expanded = unclip(&pts, UNCLIP_RATIO);
        let (ew, eh) = side_lengths(&expanded);
        if ew.min(eh) < MIN_SIZE + 2.0 {
            small_after += 1;
            continue;
        }

        // Back to original image coordinates
        let max_x = orig_w as f32 - 1.0;
        let max_y = orig_h as f32 - 1.0;
        let mapped = expanded.map(|(x, y)| {
            (
                (x / scale_x).clamp(0.0, max_x),
                (y / scale_y).clamp(0.0, max_y),
            )
        });

        boxes.push(TextBox { pts: mapped, score });
    }

    dlog!(
        "rejected: {} too small, {} below box_thresh ({}), {} too small after unclip",
        small,
        low_score,
        BOX_THRESH,
        small_after
    );

    sort_boxes(&mut boxes);
    boxes
}

/// Order 4 corners as top-left, top-right, bottom-right, bottom-left.
fn order_points(mut p: [Pt; 4]) -> [Pt; 4] {
    p.sort_by(|a, b| a.0.total_cmp(&b.0));
    let (l0, l1, r0, r1) = (p[0], p[1], p[2], p[3]);
    let (tl, bl) = if l0.1 <= l1.1 { (l0, l1) } else { (l1, l0) };
    let (tr, br) = if r0.1 <= r1.1 { (r0, r1) } else { (r1, r0) };
    [tl, tr, br, bl]
}

fn dist(a: Pt, b: Pt) -> f32 {
    ((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)).sqrt()
}

/// (width, height) of an ordered rectangle.
fn side_lengths(p: &[Pt; 4]) -> (f32, f32) {
    (dist(p[0], p[1]), dist(p[0], p[3]))
}

/// Is `p` inside the convex quad `q` (either winding)?
fn in_quad(p: Pt, q: &[Pt; 4]) -> bool {
    let (mut pos, mut neg) = (false, false);
    for i in 0..4 {
        let a = q[i];
        let b = q[(i + 1) % 4];
        let cr = (b.0 - a.0) * (p.1 - a.1) - (b.1 - a.1) * (p.0 - a.0);
        if cr > 0.0 {
            pos = true;
        }
        if cr < 0.0 {
            neg = true;
        }
    }
    !(pos && neg)
}

/// Mean probability inside the box (Paddle's `box_score_fast`).
fn box_score(probs: &[f32], w: usize, h: usize, q: &[Pt; 4]) -> f32 {
    let min_x = q
        .iter()
        .map(|p| p.0)
        .fold(f32::INFINITY, f32::min)
        .floor()
        .max(0.0) as usize;
    let max_x = (q
        .iter()
        .map(|p| p.0)
        .fold(f32::NEG_INFINITY, f32::max)
        .ceil() as usize)
        .min(w - 1);
    let min_y = q
        .iter()
        .map(|p| p.1)
        .fold(f32::INFINITY, f32::min)
        .floor()
        .max(0.0) as usize;
    let max_y = (q
        .iter()
        .map(|p| p.1)
        .fold(f32::NEG_INFINITY, f32::max)
        .ceil() as usize)
        .min(h - 1);

    let (mut sum, mut n) = (0.0_f32, 0_u32);
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            if in_quad((x as f32, y as f32), q) {
                sum += probs[y * w + x];
                n += 1;
            }
        }
    }
    if n == 0 { 0.0 } else { sum / n as f32 }
}

/// Grow the rectangle outward by `area * ratio / perimeter` on every side.
/// (Approximation of Paddle's polygon offsetting, exact for rectangles.)
fn unclip(p: &[Pt; 4], ratio: f32) -> [Pt; 4] {
    let (w, h) = side_lengths(p);
    let d = (w * h) * ratio / (2.0 * (w + h));

    let u = ((p[1].0 - p[0].0) / w, (p[1].1 - p[0].1) / w); // along the top edge
    let v = ((p[3].0 - p[0].0) / h, (p[3].1 - p[0].1) / h); // along the left edge

    let corner = |base: Pt, su: f32, sv: f32| -> Pt {
        (
            base.0 + su * d * u.0 + sv * d * v.0,
            base.1 + su * d * u.1 + sv * d * v.1,
        )
    };
    [
        corner(p[0], -1.0, -1.0),
        corner(p[1], 1.0, -1.0),
        corner(p[2], 1.0, 1.0),
        corner(p[3], -1.0, 1.0),
    ]
}

/// Top-to-bottom, left-to-right; boxes on roughly the same line sort by x.
fn sort_boxes(boxes: &mut Vec<TextBox>) {
    boxes.sort_by(|a, b| {
        a.pts[0]
            .1
            .total_cmp(&b.pts[0].1)
            .then(a.pts[0].0.total_cmp(&b.pts[0].0))
    });
    for i in 0..boxes.len().saturating_sub(1) {
        for j in (0..=i).rev() {
            let same_line = (boxes[j + 1].pts[0].1 - boxes[j].pts[0].1).abs() < 10.0;
            if same_line && boxes[j + 1].pts[0].0 < boxes[j].pts[0].0 {
                boxes.swap(j, j + 1);
            } else {
                break;
            }
        }
    }
}

// ============================================================================
// Debug helpers
// ============================================================================

fn make_debug_dir() -> Option<PathBuf> {
    let dir = std::env::temp_dir().join("snsq_debug");
    std::fs::create_dir_all(&dir).ok()?;
    Some(dir)
}

fn save_debug_images(
    dir: &PathBuf,
    img: &RgbaImage,
    probs: &[f32],
    pw: usize,
    ph: usize,
    boxes: &[TextBox],
) {
    // 0: the exact image that went into detection
    let _ = img.save(dir.join("0_input.png"));

    // 1: probability map as grayscale (white = text)
    let gray: Vec<u8> = probs
        .iter()
        .map(|&p| (p.clamp(0.0, 1.0) * 255.0) as u8)
        .collect();
    if let Some(g) = GrayImage::from_raw(pw as u32, ph as u32, gray) {
        let _ = g.save(dir.join("1_prob_map.png"));
    }

    // 3: boxes drawn on the original
    let mut canvas = img.clone();
    let red = Rgba([255, 0, 0, 255]);
    for b in boxes {
        for i in 0..4 {
            draw_line_segment_mut(&mut canvas, b.pts[i], b.pts[(i + 1) % 4], red);
        }
    }
    let _ = canvas.save(dir.join("3_boxes.png"));
}
