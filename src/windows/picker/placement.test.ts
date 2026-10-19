import { describe, expect, it } from "vitest";
import { matchLabel, OFFSET, placePanel } from "./placement";

const viewport = { width: 1000, height: 800 };
const panel = { width: 300, height: 150 };

describe("placePanel", () => {
  it("sits below-right of the cursor", () => {
    expect(placePanel({ x: 100, y: 100 }, panel, viewport)).toEqual({
      left: 100 + OFFSET,
      top: 100 + OFFSET,
    });
  });

  it("flips left and up near the edges, never covering the cursor", () => {
    const pos = placePanel({ x: 950, y: 780 }, panel, viewport);
    expect(pos).toEqual({ left: 950 - OFFSET - 300, top: 780 - OFFSET - 150 });
    expect(pos.left + panel.width).toBeLessThan(950);
    expect(pos.top + panel.height).toBeLessThan(780);
  });

  it("stays on screen on a tiny display", () => {
    const pos = placePanel({ x: 10, y: 10 }, panel, { width: 200, height: 100 });
    expect(pos.left).toBeGreaterThanOrEqual(0);
    expect(pos.top).toBeGreaterThanOrEqual(0);
  });
});

describe("matchLabel", () => {
  it("says nothing for exact names and qualifies distant ones", () => {
    expect(matchLabel(0)).toBeNull();
    expect(matchLabel(1)).toBeNull();
    expect(matchLabel(3.2)).toBe("close match");
    expect(matchLabel(14.7)).toBe("closest match");
  });
});
