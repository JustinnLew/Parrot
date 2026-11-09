import { useEffect, useRef, useState } from "react";
import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { tempDir, join } from "@tauri-apps/api/path";

type Pt = { x: number; y: number };

export default function SnipOverlay() {
  const [url, setUrl] = useState<string | null>(null);
  const [start, setStart] = useState<Pt | null>(null);
  const [end, setEnd] = useState<Pt | null>(null);
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
      if (e.key === "Escape") invoke("cancel_snip");
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  if (!url) return null;

  const rect =
    start && end
      ? {
          left: Math.min(start.x, end.x),
          top: Math.min(start.y, end.y),
          width: Math.abs(end.x - start.x),
          height: Math.abs(end.y - start.y),
        }
      : null;

  const onUp = () => {
    const img = imgRef.current;
    if (!img || !rect || rect.width < 3 || rect.height < 3) {
      setStart(null);
      setEnd(null);
      return;
    }
    // convert from CSS pixels to real image pixels (handles DPI scaling)
    const sx = img.naturalWidth / img.clientWidth;
    const sy = img.naturalHeight / img.clientHeight;
    invoke("finish_snip", {
      x: Math.round(rect.left * sx),
      y: Math.round(rect.top * sy),
      w: Math.round(rect.width * sx),
      h: Math.round(rect.height * sy),
    });
    setStart(null);
    setEnd(null);
  };

  return (
    <div
      className="fixed inset-0 cursor-crosshair select-none overflow-hidden"
      onPointerDown={(e) => {
        e.currentTarget.setPointerCapture(e.pointerId);
        setStart({ x: e.clientX, y: e.clientY });
        setEnd({ x: e.clientX, y: e.clientY });
      }}
      onPointerMove={(e) => start && setEnd({ x: e.clientX, y: e.clientY })}
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
      {rect && (
        <div
            className="pointer-events-none absolute shadow-[0_0_0_9999px_rgba(0,0,0,0.4)]"
            style={rect}
        >
            <svg className="absolute inset-0 h-full w-full overflow-visible">
            <rect width="100%" height="100%" fill="none" stroke="white" strokeWidth="1" />
            <rect
                width="100%"
                height="100%"
                fill="none"
                stroke="black"
                strokeWidth="1"
                strokeDasharray="6 6"
                className="animate-[march_0.6s_linear_infinite]"
            />
            </svg>

        </div>
        )}

      <button
        className="absolute right-2 top-2 z-10 cursor-pointer rounded bg-black/70 px-2 py-1 text-white"
        onPointerDown={(e) => e.stopPropagation()}
        onPointerUp={(e) => e.stopPropagation()}
        onClick={() => invoke("cancel_snip")}
      >
        X
      </button>
    </div>
  );
}