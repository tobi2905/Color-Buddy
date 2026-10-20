import { describe, expect, it } from "vitest";
import { acceleratorFromEvent, formatAccelerator } from "./accelerator";

const press = (
  code: string,
  mods: Partial<Record<"meta" | "ctrl" | "alt" | "shift", boolean>>,
) => ({
  code,
  metaKey: !!mods.meta,
  ctrlKey: !!mods.ctrl,
  altKey: !!mods.alt,
  shiftKey: !!mods.shift,
});

describe("acceleratorFromEvent", () => {
  it("builds accelerators from key presses", () => {
    expect(acceleratorFromEvent(press("KeyP", { ctrl: true, shift: true }))).toEqual({
      kind: "accelerator",
      accelerator: "Control+Shift+P",
    });
    expect(acceleratorFromEvent(press("Digit5", { alt: true }))).toEqual({
      kind: "accelerator",
      accelerator: "Alt+5",
    });
  });

  it("waits while only modifiers are held", () => {
    expect(acceleratorFromEvent(press("ShiftLeft", { shift: true }))).toEqual({ kind: "pending" });
  });

  it("rejects shortcuts that would block typing", () => {
    expect(acceleratorFromEvent(press("KeyC", { shift: true })).kind).toBe("invalid");
    expect(acceleratorFromEvent(press("Escape", { ctrl: true })).kind).toBe("invalid");
  });
});

describe("formatAccelerator", () => {
  it("uses symbols on macOS and words elsewhere", () => {
    expect(formatAccelerator("CommandOrControl+Shift+C", true)).toBe("⇧⌘C");
    expect(formatAccelerator("CommandOrControl+Shift+C", false)).toBe("Ctrl + Shift + C");
    expect(formatAccelerator("Control+Alt+P", true)).toBe("⌃⌥P");
    expect(formatAccelerator("Alt+5", false)).toBe("Alt + 5");
  });
});
