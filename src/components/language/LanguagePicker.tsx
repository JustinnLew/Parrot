import { useEffect, useRef, useState } from "react";
import { ArrowLeftRight, Check, ChevronDown, Languages } from "lucide-react";
import { useSettings } from "../../helpers/settings/useSettings";

const LANGS = [
  { code: "en", label: "English" },
  { code: "zh", label: "中文" },
  { code: "ko", label: "한국어" },
];

function LangSelect({
  value,
  onChange,
  label,
  disabledCode,
}: {
  value: string;
  onChange: (code: string) => void;
  label: string;
  disabledCode?: string;
}) {
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);
  const current = LANGS.find((l) => l.code === value);

  // close on outside click or Escape
  useEffect(() => {
    if (!open) return;
    const onDown = (e: MouseEvent) => {
      if (!ref.current?.contains(e.target as Node)) setOpen(false);
    };
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") setOpen(false);
    };
    document.addEventListener("mousedown", onDown);
    document.addEventListener("keydown", onKey);
    return () => {
      document.removeEventListener("mousedown", onDown);
      document.removeEventListener("keydown", onKey);
    };
  }, [open]);

  return (
    <div ref={ref} className="relative">
      <button
        type="button"
        aria-haspopup="listbox"
        aria-expanded={open}
        aria-label={label}
        title={label}
        onClick={() => setOpen((o) => !o)}
        className={`flex items-center gap-1 rounded-lg px-2 py-1 text-xs transition-colors focus-visible:outline-2 focus-visible:outline-cyan-300/60 ${
          open ? "bg-cyan-400/15 text-cyan-100" : "text-cyan-100 hover:bg-white/6"
        }`}
      >
        {current?.label ?? value}
        <ChevronDown
          size={12}
          strokeWidth={1.75}
          className={`text-cyan-200/60 transition-transform ${open ? "rotate-180" : ""}`}
        />
      </button>

      {open && (
        <div
          role="listbox"
          aria-label={label}
          className="absolute left-0 top-full z-50 mt-1 min-w-full rounded-lg border border-white/10 bg-slate-900 p-1 shadow-lg"
        >
          {LANGS.map((l) => {
            const selected = l.code === value;
            const disabled = l.code === disabledCode;
            return (
              <button
                key={l.code}
                type="button"
                role="option"
                aria-selected={selected}
                disabled={disabled}
                onClick={() => {
                  onChange(l.code);
                  setOpen(false);
                }}
                className={`flex w-full items-center justify-between gap-3 whitespace-nowrap rounded-md px-2 py-1 text-left text-xs transition-colors focus-visible:outline-2 focus-visible:outline-cyan-300/60 ${
                  selected
                    ? "bg-cyan-400/15 text-cyan-100"
                    : "text-cyan-200/80 hover:bg-white/6 hover:text-cyan-100"
                } ${disabled ? "cursor-not-allowed opacity-40 hover:bg-transparent" : ""}`}
              >
                {l.label}
                {selected && <Check size={12} strokeWidth={2} />}
              </button>
            );
          })}
        </div>
      )}
    </div>
  );
}

export function LanguagePicker() {
  const { settings, setSourceLang, setTargetLang } = useSettings();
  if (!settings) return null;

  const swap = async () => {
    const { source_lang, target_lang } = settings;
    await setSourceLang(target_lang);
    await setTargetLang(source_lang);
  };

  return (
    <div className="mr-auto flex items-center gap-0.5 rounded-lg border border-white/10 px-1.5 py-0.5 text-cyan-200/80">
      <Languages size={14} strokeWidth={1.75} className="mr-1" />
      <LangSelect
        label="Source language"
        value={settings.source_lang}
        onChange={setSourceLang}
        disabledCode={settings.target_lang}
      />
      <button
        type="button"
        onClick={swap}
        title="Swap languages"
        aria-label="Swap languages"
        className="rounded-md p-1 transition-colors hover:bg-white/6 hover:text-cyan-100 focus-visible:outline-2 focus-visible:outline-cyan-300/60"
      >
        <ArrowLeftRight size={12} strokeWidth={1.75} />
      </button>
      <LangSelect
        label="Target language"
        value={settings.target_lang}
        onChange={setTargetLang}
        disabledCode={settings.source_lang}
      />
    </div>
  );
}