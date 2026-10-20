import { invoke } from "@tauri-apps/api/core";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { mockCommands } from "../test/tauri";
import { ExportPanel } from "./ExportPanel";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/api/event", () => ({ listen: vi.fn(() => Promise.resolve(() => {})) }));

const palette = {
  name: "Sky",
  scheme: "complementary" as const,
  baseHex: "#87CEEB",
  colors: [{ hex: "#87CEEB", role: "primary" as const }],
};

beforeEach(() => {
  vi.mocked(invoke).mockReset();
  mockCommands({
    export_palette: ({ format }) => `output for ${String(format)}`,
    copy_to_clipboard: () => undefined,
    export_palette_to_file: () => "/Users/me/sky.css",
  });
});

describe("ExportPanel", () => {
  it("previews the chosen format and copies it", async () => {
    render(<ExportPanel palette={palette} />);
    expect(await screen.findByText("output for text-detailed")).toBeInTheDocument();
    await userEvent.click(screen.getByRole("radio", { name: /Names only/ }));
    expect(await screen.findByText("output for text-names")).toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: "Copy to clipboard" }));
    expect(invoke).toHaveBeenCalledWith("copy_to_clipboard", { text: "output for text-names" });
    expect(screen.getByRole("status")).toHaveTextContent("Copied to the clipboard");
  });

  it("offers every format and saves to a file", async () => {
    render(<ExportPanel palette={palette} />);
    for (const label of [
      "Detailed text",
      "Hex values only",
      "Names only",
      "Names with hex",
      "CSS variables",
      "JSON",
      "Tailwind config",
    ]) {
      expect(screen.getByRole("radio", { name: new RegExp(label) })).toBeInTheDocument();
    }
    await userEvent.click(screen.getByRole("radio", { name: /CSS variables/ }));
    await userEvent.click(screen.getByRole("button", { name: "Save as file…" }));
    expect(invoke).toHaveBeenCalledWith(
      "export_palette_to_file",
      expect.objectContaining({ format: "css" }),
    );
    expect(await screen.findByText("✓ Saved to /Users/me/sky.css")).toBeInTheDocument();
  });
});
