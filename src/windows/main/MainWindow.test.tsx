import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import MainWindow from "./MainWindow";

vi.mock("../../lib/picker", () => ({
  listHistory: vi.fn(() => Promise.resolve([])),
  onPickerFrozen: vi.fn(() => Promise.resolve(() => {})),
}));

vi.mock("../../lib/sampler", () => ({
  getSamplerStatus: vi.fn(() => new Promise(() => {})),
  onColorHovered: vi.fn(() => Promise.resolve(() => {})),
  onSamplerStatus: vi.fn(() => Promise.resolve(() => {})),
}));

describe("MainWindow", () => {
  it("renders the app title", () => {
    render(<MainWindow />);
    expect(screen.getByRole("heading", { name: "Color-Buddy" })).toBeInTheDocument();
  });
});
