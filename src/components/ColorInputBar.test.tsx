import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { mockCommands } from "../test/tauri";
import { ColorInputBar } from "./ColorInputBar";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/api/event", () => ({ listen: vi.fn(() => Promise.resolve(() => {})) }));

beforeEach(() => {
  mockCommands({
    search_color_by_name: ({ query }) =>
      String(query).startsWith("sky")
        ? [
            { name: "sky blue", hex: "#87CEEB", deltaE: 0 },
            { name: "deep sky blue", hex: "#00BFFF", deltaE: 0 },
          ]
        : [],
  });
});

describe("ColorInputBar", () => {
  it("accepts hex and rgb directly", async () => {
    const onSelect = vi.fn();
    render(<ColorInputBar onSelect={onSelect} />);
    const input = screen.getByRole("combobox");
    await userEvent.type(input, "#abc{Enter}");
    await userEvent.type(input, "rgb(135, 206, 235){Enter}");
    expect(onSelect.mock.calls).toEqual([["#AABBCC"], ["#87CEEB"]]);
  });

  it("suggests names and picks with the keyboard", async () => {
    const onSelect = vi.fn();
    render(<ColorInputBar onSelect={onSelect} />);
    const input = screen.getByRole("combobox");
    await userEvent.type(input, "sky");
    expect(await screen.findByRole("option", { name: /deep sky blue/ })).toBeInTheDocument();
    expect(input).toHaveAttribute("aria-expanded", "true");
    await userEvent.keyboard("{ArrowDown}{Enter}");
    expect(onSelect).toHaveBeenCalledWith("#00BFFF");
  });

  it("explains input it can't use", async () => {
    render(<ColorInputBar onSelect={vi.fn()} />);
    await userEvent.type(screen.getByRole("combobox"), "rgb(300, 0, 0){Enter}");
    expect(screen.getByRole("alert")).toHaveTextContent("RGB values go from 0 to 255.");
    await userEvent.clear(screen.getByRole("combobox"));
    await userEvent.type(screen.getByRole("combobox"), "zzzz");
    await new Promise((r) => setTimeout(r, 200));
    await userEvent.keyboard("{Enter}");
    expect(screen.getByRole("alert")).toHaveTextContent("No colour called “zzzz”");
  });
});
