import type { PairVerdict } from "../types/color";

const STYLE: Record<PairVerdict, { icon: string; text: string; className: string }> = {
  distinct: { icon: "✓", text: "Distinct", className: "bg-ok-bg text-ok" },
  caution: { icon: "!", text: "Caution", className: "bg-warn-bg text-warn" },
  hardToTell: { icon: "✕", text: "Hard to tell", className: "bg-bad-bg text-bad" },
};

/** Icon + word (+ ΔE as secondary detail): never relies on colour alone. */
export function VerdictBadge({ verdict, deltaE }: { verdict: PairVerdict; deltaE?: number }) {
  const s = STYLE[verdict];
  return (
    <span
      className={`inline-flex items-baseline gap-1.5 rounded px-2 py-0.5 text-sm font-semibold whitespace-nowrap ${s.className}`}
    >
      <span aria-hidden="true">{s.icon}</span>
      {s.text}
      {deltaE !== undefined && (
        <span className="font-mono text-xs font-normal opacity-80">ΔE {deltaE.toFixed(1)}</span>
      )}
    </span>
  );
}
