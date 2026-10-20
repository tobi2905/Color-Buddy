import { describe, expect, it } from "vitest";
import { palette } from "../test/fixtures";
import { mergeRegenerated } from "./palette";

describe("mergeRegenerated", () => {
  it("keeps locked colours in place and refreshes the rest", () => {
    const current = mergeRegenerated([], palette).map((c, i) =>
      i === 3 ? { ...c, hex: "#123456", locked: true } : c,
    );
    const fresh = {
      ...palette,
      colors: palette.colors.map((c) => ({ ...c, hex: c.hex.replace("#", "#0") })),
    };
    const merged = mergeRegenerated(current, fresh);
    expect(merged[3]).toMatchObject({ hex: "#123456", locked: true });
    expect(merged[2].hex).toBe(fresh.colors[2].hex);
    expect(merged.filter((c) => c.locked)).toHaveLength(1);
  });
});
