import { useState } from "react";
import NavBar from "./NavBar";
import Pairs from "./Pairs";
// import Decks from "./Decks";

export type Tab = "pairs" | "decks" | "progress";

export default function Main() {
  const [tab, setTab] = useState<Tab>("pairs");

  return (
    <main className="h-screen w-screen p-1.5 text-cyan-100">
      <div className="glass relative flex h-full w-full flex-col overflow-hidden rounded-3xl">
        <div aria-hidden className="pointer-events-none absolute -left-16 -top-16 h-56 w-56 rounded-full bg-cyan-400/25 blur-3xl" />
        <div aria-hidden className="pointer-events-none absolute -bottom-20 -right-16 h-64 w-64 rounded-full bg-indigo-500/25 blur-3xl" />

        <NavBar tab={tab} setTab={setTab} />

        <Pairs visible={tab == "pairs"} />
        {/* {tab === "decks" && <Decks />} */}
      </div>
    </main>
  );
}