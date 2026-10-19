import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import MainWindow from "./MainWindow";

vi.mock("../../lib/color", () => ({ getColorInfo: vi.fn() }));

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
