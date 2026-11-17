import type { TranslationPair } from "./Main";

export default function TranslationCard({ pair }: { pair: TranslationPair }) {
  const [main, ...alternatives] = pair.translations;
  const date = new Date(pair.created_at * 1000);

  return (
    <article className="overflow-hidden rounded-2xl border border-white/10 backdrop-blur-sm">
      <div className="grid grid-cols-2">
        <div className="bg-white/4 px-4 py-3">
          <p className="text-sm leading-relaxed text-cyan-100/80">
            {pair.source_text}
          </p>
        </div>

        <div className="bg-cyan-400/10 px-4 py-3">
            <p className="text-sm font-medium leading-relaxed text-white">{main}</p>
            {/* {pair.romanization && (
            <p className="mt-0.5 text-xs italic text-cyan-200/70">
              {pair.romanization}
            </p>
          )} */}
          {alternatives.length > 0 && (
            <ul className="mt-2.5 flex flex-wrap gap-1.5">
              {alternatives.map((alt) => (
                <li
                  key={alt}
                  className="rounded-full bg-cyan-400/15 px-2.5 py-0.5 text-xs text-cyan-100"
                >
                  {alt}
                </li>
              ))}
            </ul>
          )}
        </div>
      </div>

      <footer className="flex items-center gap-3 bg-black/20 px-4 py-2 text-[11px] text-cyan-200/60">
        <span className="font-semibold uppercase tracking-wide">
          {pair.source_lang} → {pair.target_lang}
        </span>
        {pair.note && <span className="truncate">{pair.note}</span>}
        <time dateTime={date.toISOString()} className="ml-auto shrink-0">
          {date.toLocaleString(undefined, { dateStyle: "medium", timeStyle: "short" })}
        </time>
      </footer>
    </article>
  );
}