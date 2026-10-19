import { act, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import * as api from "../../lib/sampler";
import type { HoveredEvent, SamplerStatus } from "../../types/sampler";
import SamplerDebug from "./SamplerDebug";
import { TEST_PATCHES } from "./testPatches";

vi.mock("../../lib/sampler", () => ({
  getSamplerStatus: vi.fn(),
  onColorHovered: vi.fn(),
  onSamplerStatus: vi.fn(),
  openScreenRecordingSettings: vi.fn(),
  selfTest: vi.fn(),
  setSampleSize: vi.fn(),
  startPicker: vi.fn(),
  stopPicker: vi.fn(),
}));

const idle: SamplerStatus = {
  running: false,
  size: 5,
  samplesPerSecond: 0,
  avgCaptureMs: 0,
  firstSampleMs: null,
  display: null,
  error: null,
  errorKind: null,
};

let emitHovered: (e: HoveredEvent) => void;
let emitStatus: (s: SamplerStatus) => void;

beforeEach(() => {
  vi.mocked(api.getSamplerStatus).mockResolvedValue(idle);
  vi.mocked(api.onColorHovered).mockImplementation((cb) => {
    emitHovered = cb;
    return Promise.resolve(() => {});
  });
  vi.mocked(api.onSamplerStatus).mockImplementation((cb) => {
    emitStatus = cb;
    return Promise.resolve(() => {});
  });
});

/** Render and let the initial status fetch settle so it can't overwrite emitted events. */
async function renderPanel() {
  render(<SamplerDebug />);
  await act(async () => {});
}

describe("SamplerDebug", () => {
  it("shows the hex from color-hovered events", async () => {
    await renderPanel();
    expect(screen.getByTestId("hovered-hex")).toHaveTextContent("—");
    act(() => emitHovered({ hex: "#87CEEB" }));
    expect(screen.getByTestId("hovered-hex")).toHaveTextContent("#87CEEB");
  });

  it("shows measurements and a not-sRGB note from status events", async () => {
    await renderPanel();
    act(() =>
      emitStatus({
        ...idle,
        running: true,
        samplesPerSecond: 30,
        avgCaptureMs: 2.5,
        firstSampleMs: 42.3,
        display: {
          id: 1,
          scale: 2,
          profile: "Color LCD",
          isSrgb: false,
          convertedToSrgb: true,
        },
      }),
    );
    expect(screen.getByText("30")).toBeInTheDocument();
    expect(screen.getByText("42.3 ms")).toBeInTheDocument();
    expect(screen.getByText(/not sRGB, converted to sRGB/)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Stop sampling" })).toBeInTheDocument();
  });

  it("offers the settings shortcut when Screen Recording permission is missing", async () => {
    await renderPanel();
    act(() =>
      emitStatus({
        ...idle,
        error: "Screen Recording permission is missing.",
        errorKind: "permission",
      }),
    );
    expect(screen.getByRole("alert")).toHaveTextContent("permission is missing");
    await userEvent.click(screen.getByRole("button", { name: "Open Screen Recording settings" }));
    expect(api.openScreenRecordingSettings).toHaveBeenCalled();
  });

  it("runs the self-test on all patches and labels results with text", async () => {
    vi.mocked(api.selfTest).mockResolvedValue([
      { expected: "#FF0000", size: 5, center: "#FF0000", allMatch: true, maxChannelDiff: 0 },
      { expected: "#87CEEB", size: 5, center: "#86CDEA", allMatch: false, maxChannelDiff: 1 },
    ]);
    await renderPanel();
    await userEvent.click(screen.getByRole("button", { name: "Run self-test" }));

    const sent = vi.mocked(api.selfTest).mock.calls[0][0];
    expect(sent.map((p) => p.hex)).toEqual([...TEST_PATCHES]);
    expect(await screen.findByText("✓ exact")).toBeInTheDocument();
    expect(screen.getByText("✕ mismatch")).toBeInTheDocument();
  });
});
