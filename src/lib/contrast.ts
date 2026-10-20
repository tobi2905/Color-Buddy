/** WCAG 2.x level for a contrast ratio, as text with an icon. */
export function wcagLevel(ratio: number): { level: string; ok: boolean } {
  const r = Math.floor(ratio * 100) / 100;
  if (r >= 7) return { level: "AAA", ok: true };
  if (r >= 4.5) return { level: "AA", ok: true };
  if (r >= 3) return { level: "AA large text only", ok: false };
  return { level: "Fails", ok: false };
}
