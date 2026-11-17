import type { TranslationPair } from "./Main";

export default function TranslationCard({ pair }: { pair: TranslationPair }) {
  const [main, ...alternatives] = pair.translations;
  const date = new Date(pair.created_at * 1000);

  return (
    <article className="rounded-xl border border-white/10 bg-white/5 px-3 py-2 shadow-sm backdrop-blur-sm">
      <div className="grid grid-cols-2 gap-3">
        <p className="text-sm text-cyan-100/80">{pair.source_text}</p>

        <div className="border-l border-white/10 pl-3">
          <p className="text-base font-medium text-white">{main}</p>
          {alternatives.length > 0 && (
            <p className="mt-0.5 text-xs text-cyan-200/60">
              {alternatives.join(" · ")}
            </p>
          )}
        </div>
      </div>

      <div className="mt-1.5 flex items-center gap-3 text-[11px] text-cyan-200/50">
        <span className="font-medium uppercase tracking-wide">
          {pair.source_lang} → {pair.target_lang}
        </span>
        {pair.note && <span className="truncate italic">{pair.note}</span>}
        <time dateTime={date.toISOString()} className="ml-auto shrink-0">
          {date.toLocaleString()}
        </time>
      </div>
    </article>
  );
}