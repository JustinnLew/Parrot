//! PP-OCR recognition: perspective crop -> preprocess -> ONNX -> CTC decode.

use std::error::Error;
use std::time::Instant;
use image::{imageops::FilterType, Rgba, RgbaImage};
use imageproc::geometric_transformations::{warp_into, Interpolation, Projection};
use ort::value::Tensor;

use crate::ocr::{Pt, TextCapture};

use super::det::TextBox;
use super::OcrState;

// ---- Settings (from inference.yml: RecResizeImg image_shape [3, 48, 320]) ----
const REC_H: u32 = 48;
const REC_W: u32 = 320; // minimum padded width
const REC_W_MAX: u32 = 3200; // largest width in the model's dynamic range
const DROP_SCORE: f32 = 0.5; // PaddleOCR's default

// ---- Debug switch ------------------------------------------------------------
const DEBUG: bool = true;

// ============================================================================
// Crop
// ============================================================================

/// Straighten the quad into an upright rectangle (Paddle's get_rotate_crop_image).
/// `pts` must be ordered tl, tr, br, bl.
fn crop_box(img: &RgbaImage, pts: &[Pt; 4]) -> Option<RgbaImage> {
    let dist = |a: Pt, b: Pt| ((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)).sqrt();
    let w = dist(pts[0], pts[1]).max(dist(pts[3], pts[2])).round().max(1.0) as u32;
    let h = dist(pts[0], pts[3]).max(dist(pts[1], pts[2])).round().max(1.0) as u32;

    let to = [(0.0, 0.0), (w as f32, 0.0), (w as f32, h as f32), (0.0, h as f32)];
    // Maps source-image coordinates -> crop coordinates.
    let proj = Projection::from_control_points(*pts, to)?;

    let mut out = RgbaImage::new(w, h);
    warp_into(img, proj, Interpolation::Bilinear, imageproc::geometric_transformations::Border::Constant(Rgba([255, 255, 255, 255])), &mut out);

    // Tall boxes are vertical text: rotate 90 degrees counter-clockwise.
    if h as f32 / w as f32 >= 1.5 {
        Some(image::imageops::rotate270(&out))
    } else {
        Some(out)
    }
}

// ============================================================================
// Preprocess
// ============================================================================

/// Returns planar CHW BGR data of shape [3, 48, W] and the padded width W.
fn preprocess_rec(img: &RgbaImage) -> (Vec<f32>, u32) {
    let (w, h) = img.dimensions();
    let ratio = w as f32 / h as f32;

    // Paddle: imgW = int(imgH * max(imgW/imgH, w/h)); with batch size 1 that's
    // at least 320 and grows for wide crops.
    let max_wh = (REC_W as f32 / REC_H as f32).max(ratio);
    let img_w = ((REC_H as f32 * max_wh) as u32).clamp(1, REC_W_MAX);
    let resized_w = ((REC_H as f32 * ratio).ceil() as u32).clamp(1, img_w);

    let resized = image::imageops::resize(img, resized_w, REC_H, FilterType::Triangle);

    let plane = (REC_H * img_w) as usize;
    let mut data = vec![0.0_f32; 3 * plane]; // 0.0 is also the padding value

    for (x, y, p) in resized.enumerate_pixels() {
        let bgr = [p[2], p[1], p[0]];
        let idx = y as usize * img_w as usize + x as usize;
        for c in 0..3 {
            // (v/255 - 0.5) / 0.5  ->  range [-1, 1]
            data[c * plane + idx] = (bgr[c] as f32 / 255.0 - 0.5) / 0.5;
        }
    }
    (data, img_w)
}

// ============================================================================
// CTC decode
// ============================================================================

fn char_for(idx: usize, charset: &[String]) -> &str {
    if idx < charset.len() {
        &charset[idx]
    } else if idx == charset.len() {
        " " // some exports append a space class after the dictionary
    } else {
        "?"
    }
}

/// probs is [t, c] row-major. Returns (text, mean confidence).
fn ctc_decode(probs: &[f32], t: usize, c: usize, charset: &[String]) -> (String, f32) {
    let mut text = String::new();
    let mut confs: Vec<f32> = Vec::new();
    let mut prev = 0usize;

    for step in 0..t {
        let row = &probs[step * c..(step + 1) * c];
        let (idx, &p) = row
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.total_cmp(b.1))
            .unwrap();

        // index 0 is the CTC blank; collapse repeats
        if idx != 0 && idx != prev {
            text.push_str(char_for(idx, charset));
            confs.push(p);
        }
        prev = idx;
    }

    let score = if confs.is_empty() {
        0.0
    } else {
        confs.iter().sum::<f32>() / confs.len() as f32
    };
    (text, score)
}

// ============================================================================
// Entry point
// ============================================================================

pub fn run_recognition(
    state: &OcrState,
    img: &RgbaImage,
    boxes: &[TextBox],
) -> Result<Vec<TextCapture>, Box<dyn Error>> {
    let charset = &state.rec_charset;
    let t0 = Instant::now();
    let mut lines: Vec<TextCapture> = Vec::new();

    for b in boxes {
        // 1. Crop
        let Some(crop) = crop_box(img, &b.pts) else {
            continue;
        };

        // 2. Preprocess
        let (data, in_w) = preprocess_rec(&crop);
        let shape = [1_i64, 3, REC_H as i64, in_w as i64];

        // 3. Inference
        let (steps, classes, probs): (usize, usize, Vec<f32>) = {
            let mut model = state.recognition.lock().map_err(|e| e.to_string())?;

            // if the compiler complains, add a `?` after the macro call.
            let outputs =
                model.run(ort::inputs!["x" => Tensor::from_array((shape, data))?])?;

            let (out_shape, out) = outputs[0].try_extract_tensor::<f32>()?;
            let dims: Vec<usize> = out_shape.iter().map(|&d| d as usize).collect();
            if dims.len() != 3 || dims[0] != 1 || out.len() != dims[1] * dims[2] {
                return Err(format!("unexpected recognition output shape {:?}", dims).into());
            }
            (dims[1], dims[2], out.to_vec())
        };

        // 4. Decode
        let (text, score) = ctc_decode(&probs, steps, classes, charset);

        if !text.is_empty() && score >= DROP_SCORE {
            lines.push(TextCapture { bounding_box: (b.pts), text });
        }
    }

    if DEBUG {
        for l in &lines {
            println!("[rec]   > {}", l.text);
        }
        println!("Inference in {:?}", t0.elapsed());
    }
    Ok(merge_bounding_boxes(lines))
}

#[derive(Clone, Copy)]
struct Bounds { left: f32, top: f32, right: f32, bottom: f32 }

impl Bounds {
    fn from_box(pts: &[(f32, f32); 4]) -> Self {
        let xs = pts.iter().map(|p| p.0);
        let ys = pts.iter().map(|p| p.1);
        Self {
            left: xs.clone().fold(f32::INFINITY, f32::min),
            right: xs.fold(f32::NEG_INFINITY, f32::max),
            top: ys.clone().fold(f32::INFINITY, f32::min),
            bottom: ys.fold(f32::NEG_INFINITY, f32::max),
        }
    }

    fn height(&self) -> f32 { self.bottom - self.top }

    fn union(&self, o: &Bounds) -> Bounds {
        Bounds {
            left: self.left.min(o.left),
            top: self.top.min(o.top),
            right: self.right.max(o.right),
            bottom: self.bottom.max(o.bottom),
        }
    }
}

struct Group {
    text: String,
    last: Bounds,
    all: Bounds,
}

fn join_text(a: &str, b: &str) -> String {
    if let Some(stripped) = a.strip_suffix('-') {
        return format!("{stripped}{b}"); // hyphenated wrap
    }
    let is_cjk = |c: char| matches!(c, '\u{3040}'..='\u{30ff}' | '\u{4e00}'..='\u{9fff}');
    if a.chars().last().is_some_and(is_cjk) {
        return format!("{a}{b}"); // CJK: no space
    }
    format!("{a} {b}")
}

fn merge_bounding_boxes(mut boxes: Vec<TextCapture>) -> Vec<TextCapture> {
    // reading order: top to bottom, then left to right
    boxes.sort_by(|a, b| {
        let (ax, ay) = a.bounding_box[0];
        let (bx, by) = b.bounding_box[0];
        ay.total_cmp(&by).then(ax.total_cmp(&bx))
    });

    let mut groups: Vec<Group> = Vec::new();

    for c in boxes {
        let b = Bounds::from_box(&c.bounding_box);

        let target = groups.iter_mut().find(|g| {
            let v_gap = b.top - g.last.bottom;
            let max_h = b.height().max(g.last.height());
            let min_h = b.height().min(g.last.height());
            let x_overlap = b.right.min(g.last.right) - b.left.max(g.last.left);

            v_gap < max_h * 0.6                 // close vertically
                && v_gap > -g.last.height() * 0.5 // below, not beside
                && min_h / max_h > 0.7          // similar font size
                && x_overlap > 0.0                // same column
        });

        match target {
            Some(g) => {
                g.text = join_text(&g.text, &c.text);
                g.last = b;
                g.all = g.all.union(&b);
            }
            None => groups.push(Group { text: c.text, last: b, all: b }),
        }
    }

    groups
        .into_iter()
        .map(|g| {
            let Bounds { left, top, right, bottom } = g.all;
            TextCapture {
                bounding_box: [(left, top), (right, top), (right, bottom), (left, bottom)],
                text: g.text,
            }
        })
        .collect()
}