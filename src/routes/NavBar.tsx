import { getCurrentWindow } from "@tauri-apps/api/window";
import { Minus, X } from "lucide-react";
import { Tab } from "./Main";
import { Dispatch, SetStateAction } from "react";
import { MotionConfig, motion } from "motion/react";

const TABS: { id: Tab; label: string }[] = [
  { id: "pairs", label: "Pairs" },
  { id: "decks", label: "Decks" },
  { id: "progress", label: "Progress"},
];

export default function NavBar({
  tab,
  setTab
} : {
  tab: Tab
  setTab: Dispatch<SetStateAction<Tab>>
}) {
  const win = getCurrentWindow();

  return (
    <header
      data-tauri-drag-region
      className="relative flex select-none items-center gap-6 border-b border-white/10 px-4 py-2.5 mb-2.5"
    >
      {/* Brand */}
      <div data-tauri-drag-region className="flex items-center gap-2.5">
        <div
          data-tauri-drag-region
          className="grid h-7 w-7 place-items-center rounded-full bg-linear-to-br from-cyan-300/40 to-cyan-500/10 text-sm shadow-[inset_0_1px_0_rgba(255,255,255,0.5)] ring-1 ring-white/20"
        >
          🦜
        </div>
      </div>

      {/* Tabs */}
      <MotionConfig reducedMotion="user">
        <div role="tablist" className="flex gap-0.5 rounded-xl bg-black/20 p-0.5 ring-1 ring-white/10 mr-auto overflow-x-auto scrollbar-none">
          {TABS.map((t) => {
            const active = tab === t.id;
            return (
              <button
                key={t.id}
                type="button"
                role="tab"
                aria-selected={active}
                onClick={(e) => {
                  setTab(t.id);
                  e.currentTarget.scrollIntoView({ inline: "nearest", block: "nearest", behavior: "smooth" });
                }}
                className={`relative rounded-[10px] px-3 py-1 text-xs transition-colors focus-visible:outline-2 focus-visible:outline-emerald-300/60 ${
                  active ? "font-semibold" : "text-cyan-200/70 hover:text-cyan-100"
                }`}
              >
                {active && (
                  <motion.span
                    layoutId="tab-pill"
                    className="absolute inset-0 rounded-[10px] bg-emerald-400/30 ring-1 ring-emerald-300/30"
                    transition={{ type: "spring", stiffness: 420, damping: 34 }}
                  />
                )}
                <span className="relative">{t.label}</span>
              </button>
            );
          })}
        </div>
      </MotionConfig>

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