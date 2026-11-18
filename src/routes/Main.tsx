import { useEffect, useState } from "react";
import NavBar from "./NavBar";
import { invoke } from "@tauri-apps/api/core";
import TranslationCard from "./TranslationCard";

export type TranslationPair = {
  id: number;
  created_at: number;
  note: string | null;
  source_text: string;
  source_lang: string;
  target_lang: string;
  translations: string[];
};

export default function Main() {
  const [pairs, setPairs] = useState<TranslationPair[]>([]);

  useEffect(() => {
    invoke<TranslationPair[]>("get_translation_pairs")
      .then(setPairs)
      .catch(console.error);
  }, []);

  return (
    <main className="h-screen w-screen p-1.5 text-cyan-100">
      <div className="glass relative flex h-full w-full flex-col overflow-hidden rounded-3xl">
        {/* ambient color blobs give the glass some depth */}
        <div aria-hidden className="pointer-events-none absolute -left-16 -top-16 h-56 w-56 rounded-full bg-cyan-400/25 blur-3xl" />
        <div aria-hidden className="pointer-events-none absolute -bottom-20 -right-16 h-64 w-64 rounded-full bg-indigo-500/25 blur-3xl" />

        <NavBar />
        <section className="relative flex-1 space-y-2 overflow-y-auto px-4 pb-4">
          {pairs.length === 0 ? (
            <p className="pt-8 text-center text-sm text-cyan-200/50 ">
              No translations yet...
            </p>
          ) : (
            pairs.map((pair) => <TranslationCard key={pair.id} pair={pair} />)
          )}
        </section>
      </div>
    </main>
  );
}
