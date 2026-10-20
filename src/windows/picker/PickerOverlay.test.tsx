import { act, fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import * as pickerApi from "../../lib/picker";
import * as samplerApi from "../../lib/sampler";
import type { ColorInfo } from "../../types/color";
import type { HoveredEvent } from "../../types/sampler";
import PickerOverlay from "./PickerOverlay";

vi.mock("../../lib/picker", () => ({
  freezePick: vi.fn(),
  nudgeCursor: vi.fn(),
  onPickerFrozen: vi.fn(),
  onPickerState: vi.fn(),
}));
vi.mock("../../lib/sampler", () => ({ onColorHovered: vi.fn(), stopPicker: vi.fn() }));

const info: ColorInfo = {
  hex: "#A0522D",
  rgb: { r: 160, g: 82, b: 45 },
  oklch: [0.53, 0.12, 45],
  basic: { name: "orange", hex: "#C14A09", deltaE: 7.7 },
  basicAlternative: { name: "brown", hex: "#653700", deltaE: 8.2 },
  descriptive: { name: "sienna", hex: "#A0522D", deltaE: 0 },
  precise: { name: "rust", hex: "#A83C09", deltaE: 4.1 },
  description: "medium, vivid, warm orange, bordering on brown",
  values: { hex: "#000000", rgb: "", hsl: "", oklch: "" },
};
const hover = (display: number): HoveredEvent => ({
  x: 100,
  y: 100,
  display,
  localX: 100,
  localY: 100,
  info,
  loupe: Array.from({ length: 5 }, () => Array.from({ length: 5 }, () => info.rgb)),
});

let emitHover: (e: HoveredEvent) => void;
let emitFrozen: (e: HoveredEvent) => void;
const stateHandlers: Record<string, () => void> = {};

beforeEach(() => {
  vi.clearAllMocks();
  vi.mocked(samplerApi.onColorHovered).mockImplementation((cb) => {
    emitHover = cb;
    return Promise.resolve(() => {});
  });
  vi.mocked(pickerApi.onPickerFrozen).mockImplementation((cb) => {
    emitFrozen = cb;
    return Promise.resolve(() => {});
  });
  vi.mocked(pickerApi.onPickerState).mockImplementation((name, cb) => {
    stateHandlers[name] = cb;
    return Promise.resolve(() => {});
  });
  vi.mocked(pickerApi.freezePick).mockResolvedValue(info);
  vi.mocked(pickerApi.nudgeCursor).mockResolvedValue();
  vi.mocked(samplerApi.stopPicker).mockResolvedValue();
});

describe("PickerOverlay", () => {
  it("shows the loupe and names only on its own display", () => {
    render(<PickerOverlay display={1} />);
    act(() => emitHover(hover(2)));
    expect(screen.queryByTestId("loupe")).toBeNull();
    act(() => emitHover(hover(1)));
    expect(screen.getByTestId("loupe").children).toHaveLength(25);
    expect(screen.getByTestId("descriptive")).toHaveTextContent("sienna");
    const basicLine = screen.getByText(
      (_, el) =>
        el?.tagName === "P" && !!el.textContent?.startsWith("orange, bordering on brown ·"),
    );
    expect(basicLine).toHaveTextContent("#A0522D");
    expect(screen.getByText(info.description)).toBeInTheDocument();
    expect(screen.getByText("#A0522D")).toBeInTheDocument();
  });

  it("freezes on click, shows the copied toast, and closes on the next click", async () => {
    render(<PickerOverlay display={1} />);
    act(() => emitHover(hover(1)));
    await act(async () => {
      fireEvent.mouseDown(screen.getByTestId("catcher"), { button: 0 });
    });
    expect(pickerApi.freezePick).toHaveBeenCalledTimes(1);
    act(() => emitFrozen(hover(1)));
    expect(screen.getByTestId("toast")).toHaveTextContent("✓ Copied #A0522D (sienna)");
    // Live updates are ignored while frozen.
    act(() => emitHover({ ...hover(1), info: { ...info, hex: "#000000" } }));
    expect(screen.getByTestId("toast")).toHaveTextContent("#A0522D");
    await act(async () => {
      fireEvent.mouseDown(screen.getByTestId("catcher"), { button: 0 });
    });
    expect(samplerApi.stopPicker).toHaveBeenCalledTimes(1);
  });

  it("handles Esc, Enter and arrow keys", async () => {
    render(<PickerOverlay display={1} />);
    act(() => emitHover(hover(1)));
    await act(async () => {
      fireEvent.keyDown(window, { key: "ArrowRight" });
      fireEvent.keyDown(window, { key: "ArrowUp", shiftKey: true });
      fireEvent.keyDown(window, { key: "Enter" });
      fireEvent.keyDown(window, { key: "Escape" });
    });
    expect(pickerApi.nudgeCursor).toHaveBeenCalledWith(1, 0);
    expect(pickerApi.nudgeCursor).toHaveBeenCalledWith(0, -10);
    expect(pickerApi.freezePick).toHaveBeenCalledTimes(1);
    expect(samplerApi.stopPicker).toHaveBeenCalledTimes(1);
  });

  it("returns to live mode when resumed", () => {
    render(<PickerOverlay display={1} />);
    act(() => emitFrozen(hover(1)));
    expect(screen.getByTestId("toast")).toBeInTheDocument();
    act(() => stateHandlers["picker-resumed"]());
    expect(screen.queryByTestId("toast")).toBeNull();
  });
});
