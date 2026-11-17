import NavBar from "./NavBar";

export default function Main() {
  return (
    <main className="h-screen w-screen p-1.5 text-cyan-100">
      <div className="glass relative flex h-full w-full flex-col overflow-hidden rounded-3xl">
        {/* ambient color blobs give the glass some depth */}
        <div aria-hidden className="pointer-events-none absolute -left-16 -top-16 h-56 w-56 rounded-full bg-cyan-400/25 blur-3xl" />
        <div aria-hidden className="pointer-events-none absolute -bottom-20 -right-16 h-64 w-64 rounded-full bg-indigo-500/25 blur-3xl" />

        <NavBar />
        <section className="relative flex-1 overflow-y-auto px-4 pb-4">
          {/* translation pairs list goes here */}
        </section>
      </div>
    </main>
  );
}