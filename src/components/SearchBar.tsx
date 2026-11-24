import { RefObject, useMemo, useState } from "react";
import { TranslationPair } from "../routes/Main";
import { invoke } from "@tauri-apps/api/core";

const MAX_RESULTS = 8;

export default function SearchBar({
    pairs,
    scrollRef,
}: {
    pairs: TranslationPair[];
    scrollRef: RefObject<HTMLElement | null>;
}) {
    const [query, setQuery] = useState("");
    const [active, setActive] = useState(0);
    const [busy, setBusy] = useState(false);
    const trimmed = query.trim();

    const onTranslate = async (text: string) => {
        await invoke("snip_confirm", {
            lines: [text],
        })
    }

    const matches = useMemo(() => {
        if (!trimmed) return [];
        const q = trimmed.toLowerCase();
        return pairs
            .filter(
                (p) =>
                    p.source_text.toLowerCase().includes(q)
            )
            .slice(0, MAX_RESULTS);
    }, [pairs, trimmed]);

    const total = matches.length + 1; // +1 for the Translate option

    function scrollToPair(id: number) {
        const el = document.getElementById(`pair-${id}`);
        const container = scrollRef.current;
        if (!el || !container) return;

        const elRect = el.getBoundingClientRect();
        const cRect = container.getBoundingClientRect();
        const top =
            container.scrollTop +
            (elRect.top - cRect.top) -
            (container.clientHeight - elRect.height) / 2;

        container.scrollTo({ top, behavior: "smooth" });

        const target = (el.firstElementChild as HTMLElement) ?? el;
        target.classList.add("ring-2", "ring-purple-300/80");
        setTimeout(() => target.classList.remove("ring-2", "ring-purple-300/80"), 3000);
    }

    async function runTranslate() {
        if (!trimmed || busy) return;
        setBusy(true);
        try {
            await onTranslate(trimmed);
            setQuery("");
        } finally {
            setBusy(false);
        }
    }

    function choose(index: number) {
        if (index === 0) return runTranslate();
        scrollToPair(matches[index - 1].id);
    }

    function onKeyDown(e: React.KeyboardEvent) {
        if (e.key === "ArrowDown") {
            e.preventDefault();
            setActive((a) => (a + 1) % total);
        } else if (e.key === "ArrowUp") {
            e.preventDefault();
            setActive((a) => (a - 1 + total) % total);
        } else if (e.key === "Enter") {
            choose(active);
        } else if (e.key === "Escape") {
            setQuery("");
        }
    }

    return (
        <div className="flex px-5 mt-1 mb-3 flex-col">
            <input
                type="text"
                value={query}
                className="flex-1 border border-white/40 focus:border-purple-400/70
                ring-0 focus:ring focus:ring-purple-400/30 transition-all duration-200
                rounded-md py-1.5 px-1.5 text-sm focus:outline-hidden"
                placeholder="Search..."
                onChange={(e) => {
                    setQuery(e.target.value);
                    setActive(0);
                }}
                onKeyDown={onKeyDown}
            />
            {trimmed !== "" && (
                <div className="mt-1 max-h-64 overflow-y-auto rounded-md border border-white/20">
                    <SearchOption
                        active={active === 0}
                        onHover={() => setActive(0)}
                        onClick={() => choose(0)}
                        primary={busy ? "Translating…" : `Translate "${trimmed}"`}
                    />
                    {matches.map((p, i) => (
                        <SearchOption
                            key={p.id}
                            active={active === i + 1}
                            onHover={() => setActive(i + 1)}
                            onClick={() => choose(i + 1)}
                            primary={p.source_text}
                        />
                    ))}
                </div>
            )}
            <hr className="border-b border-y-0 border-white/30 pb-2 mb-1 mt-2" />
        </div>
    );
}

function SearchOption({
    primary,
    secondary,
    active,
    onHover,
    onClick,
}: {
    primary: string;
    secondary?: string;
    active: boolean;
    onHover: () => void;
    onClick: () => void;
}) {
    return (
        <button
            type="button"
            onMouseEnter={onHover}
            onClick={onClick}
            className={`flex w-full items-center justify-between gap-3 px-2 py-1.5 text-left text-sm ${
                active ? "bg-white/10" : ""
            }`}
        >
            <span className="truncate">{primary}</span>
            {secondary && <span className="truncate opacity-60">{secondary}</span>}
        </button>
    );
}