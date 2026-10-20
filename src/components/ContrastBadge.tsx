import { wcagLevel } from "../lib/contrast";

export function ContrastBadge({ ratio }: { ratio: number }) {
  const { level, ok } = wcagLevel(ratio);
  return (
    <span className="inline-flex items-baseline gap-1.5 text-sm whitespace-nowrap">
      <span className="font-mono">{ratio.toFixed(2)}:1</span>
      <span className={`rounded px-1.5 font-semibold ${ok ? "bg-ok-bg text-ok" : "bg-raised"}`}>
        <span aria-hidden="true">{ok ? "✓ " : "· "}</span>
        {level}
      </span>
    </span>
  );
}
