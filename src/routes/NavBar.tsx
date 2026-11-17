import { getCurrentWindow } from "@tauri-apps/api/window";
import { Minus, X } from "lucide-react";

export default function NavBar() {
  const win = getCurrentWindow();

  return (
    <header
      data-tauri-drag-region
      className="relative flex select-none items-center gap-3 border-b border-white/10 px-4 py-2.5"
    >
      {/* Brand */}
      <div data-tauri-drag-region className="flex items-center gap-2.5">
        <div
          data-tauri-drag-region
          className="grid h-7 w-7 place-items-center rounded-full bg-linear-to-br from-cyan-300/40 to-cyan-500/10 text-sm shadow-[inset_0_1px_0_rgba(255,255,255,0.5)] ring-1 ring-white/20"
        >
          🦜
        </div>
        <span
          data-tauri-drag-region
          className="text-sm font-semibold tracking-wide text-cyan-100"
        >
          Parrot
        </span>
      </div>

      {/* Drag spacer */}
      <div data-tauri-drag-region className="flex-1 self-stretch" />

      {/* Window controls */}
      <div className="flex items-center gap-1.5">
        <button
          aria-label="Minimize"
          onClick={() => win.minimize()}
          className="grid h-7 w-7 cursor-pointer place-items-center rounded-full text-cyan-200/80 transition hover:bg-white/15 hover:text-white active:scale-90"
        >
          <Minus size={14} strokeWidth={2.5} />
        </button>
        <button
          aria-label="Close"
          onClick={() => win.close()}
          className="grid h-7 w-7 cursor-pointer place-items-center rounded-full text-cyan-200/80 transition hover:bg-red-500/70 hover:text-white active:scale-90"
        >
          <X size={14} strokeWidth={2.5} />
        </button>
      </div>
    </header>
  );
}