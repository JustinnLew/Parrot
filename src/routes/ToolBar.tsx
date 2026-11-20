import type { LucideIcon } from "lucide-react";
import { Eye, EyeOff } from "lucide-react";
import { useSettings } from "../helpers/settings/useSettings";
import { LanguagePicker } from "../components/language/LanguagePicker";

/** Slim controls row that sits under the NavBar. Add more <ToolButton>s here. */
export default function ToolBar({
  showReadings,
  onToggleReadings,
}: {
  showReadings: boolean;
  onToggleReadings: () => void;
}) {
  const { settings } = useSettings();

  return (
    <div className="relative mx-4 my-2 flex items-center justify-end gap-1 border-b border-white/10 pb-2">
      {settings &&
        <LanguagePicker/>
      }
      <ToolButton
        icon={showReadings ? Eye : EyeOff}
        label="Readings"
        pressed={showReadings}
        onClick={onToggleReadings}
      />
    </div>
  );
}

function ToolButton({
  icon: Icon,
  label,
  pressed,
  onClick,
}: {
  icon: LucideIcon;
  label: string;
  pressed: boolean;
  onClick: () => void;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      aria-pressed={pressed}
      title={label}
      className={`flex items-center gap-1.5 rounded-lg px-2 py-1 text-xs transition-colors focus-visible:outline-2 focus-visible:outline-cyan-300/60 ${
        pressed
          ? "bg-cyan-400/15 text-cyan-100"
          : "text-cyan-200/80 hover:bg-white/10 hover:text-cyan-100"
      }`}
    >
      <Icon size={14} strokeWidth={1.75} />
      {label}
    </button>
  );
}