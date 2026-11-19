import { useId, useState } from "react";
import { ChevronDown, Play, X } from "lucide-react";
import { getReadingProvider } from "../helpers/readings";
import type { TranslationPair } from "./Main";
import { speak } from "../helpers/tts/webSpeech";

function compactTime(ts: number, now = Date.now()): string {
  const secs = Math.max(0, Math.floor(now / 1000 - ts));
  if (secs < 60) return "now";
  const mins = Math.floor(secs / 60);
  if (mins < 60) return `${mins}m`;
  const hours = Math.floor(mins / 60);
  if (hours < 24) return `${hours}h`;
  const days = Math.floor(hours / 24);
  if (days < 7) return `${days}d`;

  const date = new Date(ts * 1000);
  const sameYear = date.getFullYear() === new Date(now).getFullYear();
  return date.toLocaleDateString(
    undefined,
    sameYear
      ? { day: "numeric", month: "short" }
      : { day: "numeric", month: "short", year: "2-digit" }
  );
}

export default function TranslationCard({
  pair,
  showReadings,
  onDelete,
}: {
  pair: TranslationPair;
  showReadings: boolean;
  onDelete: (id: number) => void;
}) {
  const [main, ...alternatives] = pair.translations;
  const provider = getReadingProvider(pair.target_lang);
  const reading = main && showReadings ? provider?.annotate(main) : undefined;
  const [expanded, setExpanded] = useState(false);
  const panelId = useId();
  const date = new Date(pair.created_at * 1000);
  const hasAlts = alternatives.length > 0;

  return (
    <article className="overflow-hidden rounded-2xl border border-white/10 backdrop-blur-sm">
      <div className="grid grid-cols-2 divide-x divide-white/10">
        <div className="flex flex-col gap-2 bg-white/4 px-4 py-3">
          <p className="text-sm leading-relaxed text-cyan-100/80">{pair.source_text}</p>
          <div className="flex items-center gap-2 text-[11px] text-cyan-200/50 mt-auto">
            <span className="font-semibold uppercase tracking-wide mr-auto">
              {pair.source_lang} → {pair.target_lang}
            </span>
            <time
              dateTime={date.toISOString()}
              title={date.toLocaleString(undefined, { dateStyle: "medium", timeStyle: "short" })}
            >
              {compactTime(pair.created_at)}
            </time>
            {pair.note && <span className="truncate">{pair.note}</span>}
          </div>
        </div>

        <div className="flex flex-col bg-cyan-400/10">
          <div className="px-4 py-3">
            <div className="flex items-start justify-between gap-1">
              <p className="min-w-0 text-sm leading-relaxed text-white">{main}</p>
              <div className="flex gap-2">
                <button
                    type="button"
                    onClick={() => speak(main, pair.target_lang)}
                    aria-label="Audio translation"
                    title="Audio"
                    className="shrink-0 rounded-md p-1 text-cyan-200/40 transition-colors hover:bg-emerald-400/15 hover:text-emerald-300 focus-visible:outline-2 focus-visible:outline-emerald-300/60"
                >
                    <Play size={14} strokeWidth={1.75} />
                </button>
              <button
                type="button"
                onClick={() => onDelete(pair.id)}
                aria-label="Delete translation"
                title="Delete"
                className="shrink-0 rounded-md p-1 text-cyan-200/40 transition-colors hover:bg-rose-400/15 hover:text-rose-300 focus-visible:outline-2 focus-visible:outline-rose-300/60"
              >
                <X size={14} strokeWidth={1.75} />
              </button>
              </div>
            </div>

            {reading && <p className="mt-0.5 text-xs italic text-cyan-200/70">{reading}</p>}

            {hasAlts && (
              <button
                type="button"
                onClick={() => setExpanded((v) => !v)}
                aria-expanded={expanded}
                aria-controls={panelId}
                className="-ml-1 mt-2 flex w-fit items-center gap-1 rounded-md px-1 py-0.5 text-[11px] text-cyan-200/60 transition-colors hover:bg-white/10 hover:text-cyan-100 focus-visible:outline-2 focus-visible:outline-cyan-300/60"
              >
                <ChevronDown
                  size={13}
                  strokeWidth={1.75}
                  className={`transition-transform duration-200 ${expanded ? "rotate-180" : ""}`}
                />
                {alternatives.length} {alternatives.length === 1 ? "alternative" : "alternatives"}
              </button>
            )}
          </div>

          {hasAlts && (
            <div
              id={panelId}
              aria-hidden={!expanded}
              className={`grid transition-[grid-template-rows] duration-200 ease-out motion-reduce:transition-none ${
                expanded ? "grid-rows-[1fr]" : "grid-rows-[0fr]"
              }`}
            >
              <div className="min-h-0 overflow-hidden">
                <ul className="space-y-2 border-t border-white/10 bg-cyan-400/5 px-4 py-2.5">
                  {alternatives.map((alt, i) => {
                    const altReading = showReadings ? provider?.annotate(alt) : undefined;
                    return (
                        <div key={`${i}:${alt}`} className="flex justify-between">
                            <li>
                                <p className="text-sm leading-snug text-white/90">{alt}</p>
                                {altReading && (
                                    <p className="mt-0.5 text-xs italic text-cyan-200/70">{altReading}</p>
                                )}
                            </li>
                            <button
                                type="button"
                                onClick={() => speak(alt, pair.target_lang)}
                                aria-label="Audio translation"
                                title="Audio"
                                className="h-fit rounded-md p-1 text-cyan-200/40 transition-colors hover:bg-emerald-400/15 hover:text-emerald-300"
                                >
                                <Play size={14} strokeWidth={1.75} />
                            </button>
                        </div>
                    );
                  })}
                </ul>
              </div>
            </div>
          )}
        </div>
      </div>
    </article>
  );
}