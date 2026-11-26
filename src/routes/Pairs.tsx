import { useCallback, useEffect, useReducer, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import ToolBar from "./ToolBar";
import TranslationCard from "./TranslationCard";
import { AnimatePresence, MotionConfig, motion } from "motion/react";
import SearchBar from "../components/SearchBar";

export type TranslationPair = {
  id: number;
  created_at: number;
  note: string | null;
  source_text: string;
  source_lang: string;
  target_lang: string;
  translations: string[];
};

const READINGS_KEY = "parrot:readings-visible";

function useReadingsVisible() {
  const [visible, setVisible] = useState(() => {
    try {
      return localStorage.getItem(READINGS_KEY) !== "0";
    } catch {
      return true;
    }
  });

  useEffect(() => {
    try {
      localStorage.setItem(READINGS_KEY, visible ? "1" : "0");
    } catch {
      /* storage unavailable: ignore */
    }
  }, [visible]);

  return [visible, setVisible] as const;
}

export default function Pairs({ visible }: { visible: boolean }) {
      const [pairs, setPairs] = useState<TranslationPair[]>([]);
      const [hasMore, setHasMore] = useState(true);
      const [showReadings, setShowReadings] = useReadingsVisible();

      // re-render once a minute so "5m" style timestamps stay fresh
      const [, tick] = useReducer((n: number) => n + 1, 0);
      useEffect(() => {
        const t = window.setInterval(tick, 60_000);
        return () => window.clearInterval(t);
      }, []);

      /* ---------- pagination ---------- */

      const cursorRef = useRef<number | null>(null);
      const loadingRef = useRef(false);
      const scrollRef = useRef<HTMLElement>(null);
      const sentinelRef = useRef<HTMLDivElement>(null);
      const liveIds = useRef<Set<number>>(new Set());

      const loadMore = useCallback(async () => {
        if (loadingRef.current) return;
        loadingRef.current = true;
        try {
          const page = await invoke<TranslationPair[]>("get_translation_pairs", {
            cursor: cursorRef.current,
          });
          if (page.length > 0) {
            cursorRef.current = page[page.length - 1].id;
            setPairs((prev) => {
              const seen = new Set(prev.map((p) => p.id));
              return [...prev, ...page.filter((p) => !seen.has(p.id))];
            });
          }
          if (page.length === 0) setHasMore(false);
        } catch (e) {
          console.error(e);
        } finally {
          loadingRef.current = false;
        }
      }, []);

      useEffect(() => {
        const root = scrollRef.current;
        const el = sentinelRef.current;
        if (!root || !el || !hasMore) return;

        const io = new IntersectionObserver(
          (entries) => {
            if (entries[0].isIntersecting) loadMore();
          },
          { root, rootMargin: "200px" }
        );
        io.observe(el);
        return () => io.disconnect();
      }, [hasMore, loadMore, pairs.length]);

      /* ---------- live additions ---------- */

      useEffect(() => {
        const unlisten = listen<TranslationPair[]>("translations-added", (event) => {
          event.payload.forEach((p) => liveIds.current.add(p.id));
          setPairs((prev) => {
            const seen = new Set(prev.map((p) => p.id));
            const fresh = event.payload.filter((p) => !seen.has(p.id));
            return [...fresh, ...prev];
          });
        });

        return () => {
          unlisten.then((fn) => fn());
        };
      }, []);

      /* ---------- delete ---------- */

      const deletePair = useCallback(async (id: number) => {
        try {
          await invoke("delete_translation_pairs", { pairIds: [id] });
          setPairs((prev) => prev.filter((p) => p.id !== id));
        } catch (e) {
          console.error("Failed to delete pair", e);
        }
      }, []);

      /* ---------- render ---------- */

      return (
        <div className={`flex min-h-0 flex-1 flex-col ${visible ? "" : "hidden"}`}>

            <ToolBar
              showReadings={showReadings}
              onToggleReadings={() => setShowReadings((v) => !v)}
            />
            <SearchBar pairs={pairs} scrollRef={scrollRef} />

          <MotionConfig reducedMotion="user">
            <section
              ref={scrollRef}
              className="scroll-glass relative flex-1 space-y-2 overflow-y-auto px-4 pb-4 overflow-x-hidden"
            >
              <AnimatePresence initial={false} mode="popLayout">
                {pairs.map((pair) => (
                  <motion.div
                    id={`pair-${pair.id}`}
                    key={pair.id}
                    layout="position"
                    initial={
                      liveIds.current.has(pair.id)
                        ? { opacity: 0, y: -16, scale: 0.96 }
                        : { opacity: 0 }
                    }
                    animate={{ opacity: 1, y: 0, scale: 1 }}
                    exit={{ opacity: 0, x: 48, scale: 0.92, transition: { duration: 0.18 } }}
                    transition={{ type: "spring", stiffness: 420, damping: 34 }}
                  >
                    <TranslationCard
                      pair={pair}
                      showReadings={showReadings}
                      onDelete={deletePair}
                    />
                  </motion.div>
                ))}
              </AnimatePresence>

              {!hasMore && pairs.length === 0 && (
                <p className="pt-8 text-center text-sm text-cyan-200/50">
                  No translations yet...
                </p>
              )}

              {hasMore && <div ref={sentinelRef} className="h-px" />}
            </section>
          </MotionConfig>
        </div>
      );
}