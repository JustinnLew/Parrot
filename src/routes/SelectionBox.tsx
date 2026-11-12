type Rect = { left: number; top: number; width: number; height: number };

export function SelectionBox({ rect, state }: { rect: Rect | null; state: string }) {
  const processing = state === "processing";
  if (!rect) return null;

  return (
    <div
      className="pointer-events-none absolute shadow-[0_0_0_9999px_rgba(0,0,0,0.4)]"
      style={rect}
    >
      <svg className="absolute inset-0 h-full w-full overflow-visible">
        <defs>
          <linearGradient id="snip-rainbow" x1="0" y1="0" x2="1" y2="1">
            <stop offset="0%" stopColor="#ff3b30" />
            <stop offset="20%" stopColor="#ff9500" />
            <stop offset="40%" stopColor="#ffd60a" />
            <stop offset="60%" stopColor="#34c759" />
            <stop offset="80%" stopColor="#0a84ff" />
            <stop offset="100%" stopColor="#bf5af2" />
            {/* spins the gradient so the colours travel around the border */}
            <animateTransform
              attributeName="gradientTransform"
              type="rotate"
              from="0 0.5 0.5"
              to="360 0.5 0.5"
              dur="2s"
              repeatCount="indefinite"
            />
          </linearGradient>
        </defs>

        {processing ? (
          <rect
            width="100%"
            height="100%"
            fill="none"
            stroke="url(#snip-rainbow)"
            strokeWidth="3"
          />
        ) : (
          <>
            <rect width="100%" height="100%" fill="none" stroke="white" strokeWidth="1" />
            <rect
              width="100%"
              height="100%"
              fill="none"
              stroke="black"
              strokeWidth="1"
              strokeDasharray="6 6"
              className="animate-[march_0.6s_linear_infinite]"
            />
          </>
        )}
      </svg>
    </div>
  );
}