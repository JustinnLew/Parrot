import { useEffect, useRef, useState } from "react";
import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { tempDir, join } from "@tauri-apps/api/path";
import { SelectionBox } from "./SelectionBox";

type Pt = [number, number];
type TextCapture = { boundingBox: [Pt, Pt, Pt, Pt]; text: string };
type SnippingState = "drawing" | "processing" | "approval";

// Are the OCR boxes relative to the cropped region (true) or the full screenshot (false)?
const BOXES_RELATIVE_TO_CROP = true;

export default function SnipOverlay() {
  const [url, setUrl] = useState<string | null>(null);
  const [start, setStart] = useState<Pt | null>(null);
  const [end, setEnd] = useState<Pt | null>(null);
  const [textCapture, setTextCapture] = useState<TextCapture[]>([]);
  const [state, setState] = useState<SnippingState>("drawing");
  const [hovered, setHovered] = useState<number | null>(null);
  // Scale factors (image px per CSS px), saved when the snip is made
  const [scale, setScale] = useState<{ sx: number; sy: number }>({ sx: 1, sy: 1 });
  const imgRef = useRef<HTMLImageElement>(null);

  useEffect(() => {
    (async () => {
      const dir = await tempDir();
      const path = await join(dir, "snsq_snip.png");
      setUrl(`${convertFileSrc(path)}?t=${Date.now()}`);
    })();
  }, []);

  // Esc cancels
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") invoke("snip_cancel");
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  const rect =
    start && end
      ? {
          left: Math.min(start[0], end[0]),
          top: Math.min(start[1], end[1]),
          width: Math.abs(end[0] - start[0]),
          height: Math.abs(end[1] - start[1]),
        }
      : null;

  const reset = () => {
    setStart(null);
    setEnd(null);
    setTextCapture([]);
    setState("drawing");
  };

  const onUp = () => {
    if (state !== "drawing") return;
    const img = imgRef.current;
    if (!img || !rect || rect.width < 3 || rect.height < 3) {
      reset();
      return;
    }
    setState("processing");

    const sx = img.naturalWidth / img.clientWidth;
    const sy = img.naturalHeight / img.clientHeight;
    setScale({ sx, sy });

    invoke<TextCapture[]>("snip_process", {
      x: Math.round(rect.left * sx),
      y: Math.round(rect.top * sy),
      w: Math.round(rect.width * sx),
      h: Math.round(rect.height * sy),
    })
      .then((lines) => {
        setTextCapture(lines);
        setState("approval");
      })
      .catch((err) => {
        console.error("snip_process failed:", err);
        reset();
      });
  };

  // Image px (from OCR) -> CSS px on screen
  const toScreen = (p: Pt): Pt => {
    const offX = BOXES_RELATIVE_TO_CROP && rect ? rect.left : 0;
    const offY = BOXES_RELATIVE_TO_CROP && rect ? rect.top : 0;
    return [ offX + p[0] / scale.sx, offY + p[1] / scale.sy ];
  };

  if (!url) return null;
  return (
    <div
      className="fixed inset-0 cursor-crosshair select-none overflow-hidden"
      onPointerDown={(e) => {
        if (state !== "drawing") return;
        e.currentTarget.setPointerCapture(e.pointerId);
        setStart([e.clientX, e.clientY ]);
        setEnd([e.clientX, e.clientY ]);
      }}
      onPointerMove={(e) =>
        start && state === "drawing" && setEnd([e.clientX, e.clientY ])
      }
      onPointerUp={onUp}
    >
      <img
        ref={imgRef}
        src={url}
        className="absolute inset-0 h-full w-full object-fill"
        draggable={false}
      />

      {!rect && (
        <div className="pointer-events-none absolute inset-0 bg-black/40" />
      )}

      {/* selection box */}
      <SelectionBox rect={rect} state={state} />

      {/* OCR overlay */}
      {state === "approval" &&
        textCapture.map((t, i) => {
          if (!t?.boundingBox || t.boundingBox.length !== 4) return null;
          const pts = t.boundingBox.map(toScreen);
          const left = Math.min(...pts.map((p) => p[0]));
          const top = Math.min(...pts.map((p) => p[1]));
          const width = Math.max(...pts.map((p) => p[0])) - left;
          const height = Math.max(...pts.map((p) => p[1])) - top;

          return (
            <div
              key={i}
              className="
                absolute flex cursor-pointer items-center overflow-hidden
                rounded-sm border px-0.5 text-white"
              style={{
                left,
                top,
                width,
                height,
              }}
              onPointerDown={(e) => e.stopPropagation()}
              onPointerEnter={() => setHovered(i)}
              onPointerLeave={() => setHovered(null)}
              onClick={() => navigator.clipboard.writeText(t.text)}
            >
            </div>
          );
        })}

      {/* tooltip showing the hovered text */}
      {state === "approval" && hovered !== null && textCapture[hovered] && (() => {
        const p = toScreen(textCapture[hovered].boundingBox[3]);
        return (
          <div
            className="pointer-events-none absolute z-10 max-w-xs rounded bg-black/80 px-2 py-1 text-sm text-white"
            style={{ left: p[0], top: p[1] + 4 }}
          >
            {textCapture[hovered].text}
          </div>
        );
      })()}

      <div className="
          absolute left-1/2 top-4 z-10 -translate-x-1/2
          flex items-center gap-1 p-1.5
          rounded-2xl
          bg-white/10 backdrop-blur-xl backdrop-saturate-150
          border border-white/30
          shadow-[0_8px_32px_rgba(0,0,0,0.35),inset_0_1px_0_rgba(255,255,255,0.4)]
          ring-1 ring-black/10
        ">
        <button
          className="z-10 cursor-pointer rounded px-2 py-1 text-white"
          onPointerDown={(e) => e.stopPropagation()}
          onPointerUp={(e) => e.stopPropagation()}
          onClick={() => invoke("snip_confirm", {
            lines: textCapture.map((t) => t.text)
          })}
        >
          Confirm
        </button>
        <button
          className="z-10 cursor-pointer rounded px-2 py-1 text-white"
          onPointerDown={(e) => e.stopPropagation()}
          onPointerUp={(e) => e.stopPropagation()}
          onClick={() => reset()}
        >
          Reset
        </button>
        <button
          className="z-10 cursor-pointer rounded px-2 py-1 text-white"
          onPointerDown={(e) => e.stopPropagation()}
          onPointerUp={(e) => e.stopPropagation()}
          onClick={() => invoke("snip_cancel")}
        >
          Exit
        </button>
      </div>
    </div>
  );
}