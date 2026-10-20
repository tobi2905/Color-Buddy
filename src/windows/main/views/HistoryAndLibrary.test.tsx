import { invoke } from "@tauri-apps/api/core";
import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { useApp } from "../../../store/app";
import { DEFAULT_SETTINGS, useSettings } from "../../../store/settings";
import { skyBlue, skyBlueDetails } from "../../../test/fixtures";
import { mockCommands } from "../../../test/tauri";
import { ColorDetailsView } from "./ColorDetailsView";
import { HistoryView } from "./HistoryView";
import { LibraryView } from "./LibraryView";
import { SettingsView } from "./SettingsView";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/api/event", () => ({ listen: vi.fn(() => Promise.resolve(() => {})) }));

const saved = {
  id: 7,
  name: "Sky afternoon",
  createdAt: Date.UTC(2026, 9, 20),
  baseHex: "#87CEEB",
  scheme: "complementary",
  colors: [
    { hex: "#F0FAFF", role: "background", locked: false },
    { hex: "#87CEEB", role: "primary", locked: true },
  ],
};

beforeEach(() => {
  vi.mocked(invoke).mockReset();
  useSettings.setState({ ...DEFAULT_SETTINGS, onboarded: true });
  useApp.setState({ currentHex: "#87CEEB", view: "color", openedPalette: null });
  mockCommands({
    list_history: () => [
      { id: 2, hex: "#87CEEB", pickedAt: Date.now(), source: "picker" },
      { id: 1, hex: "#FF0000", pickedAt: Date.now() - 60_000, source: "picker" },
    ],
    get_color_info: () => skyBlue,
    get_color_details: () => skyBlueDetails,
    list_palettes: () => [saved],
  });
});

describe("History", () => {
  it("removes one entry or clears all after confirming", async () => {
    render(<HistoryView />);
    await userEvent.click(
      await screen.findByRole("button", { name: "Remove #FF0000 from history" }),
    );
    expect(invoke).toHaveBeenCalledWith("delete_history_entry", { id: 1 });
    expect(screen.queryByRole("button", { name: "Remove #FF0000 from history" })).toBeNull();

    await userEvent.click(screen.getByRole("button", { name: "Clear history" }));
    expect(invoke).not.toHaveBeenCalledWith("clear_history", expect.anything());
    const confirm = screen.getByRole("group", { name: "Remove every picked colour from history?" });
    await userEvent.click(within(confirm).getByRole("button", { name: /Clear history/ }));
    expect(invoke).toHaveBeenCalledWith("clear_history");
    expect(await screen.findByText(/Nothing picked yet/)).toBeInTheDocument();
  });
});

describe("Saved palettes", () => {
  it("lists palettes with their values as text and opens one", async () => {
    render(<LibraryView />);
    expect(await screen.findByRole("heading", { name: "Sky afternoon" })).toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: "Copy primary hex #87CEEB" }));
    expect(invoke).toHaveBeenCalledWith("copy_to_clipboard", { text: "#87CEEB" });
    expect(screen.getByRole("button", { name: "Copy background hex #F0FAFF" })).toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: "Open" }));
    expect(useApp.getState()).toMatchObject({ view: "palettes", openedPalette: { id: 7 } });
  });

  it("renames and deletes after confirming", async () => {
    render(<LibraryView />);
    await userEvent.click(await screen.findByRole("button", { name: "Rename" }));
    const input = screen.getByLabelText("New name for Sky afternoon");
    await userEvent.clear(input);
    await userEvent.type(input, "Evening sky{Enter}");
    expect(invoke).toHaveBeenCalledWith("rename_palette", { id: 7, name: "Evening sky" });
    expect(await screen.findByRole("heading", { name: "Evening sky" })).toBeInTheDocument();

    await userEvent.click(screen.getByRole("button", { name: "Delete" }));
    await userEvent.click(screen.getByRole("button", { name: /Delete palette/ }));
    expect(invoke).toHaveBeenCalledWith("delete_palette", { id: 7 });
    expect(await screen.findByText(/No saved palettes yet/)).toBeInTheDocument();
  });
});

describe("Value format setting", () => {
  it("shows and copies values in the chosen format", async () => {
    useSettings.setState({ valueFormat: "rgb" });
    mockCommands({
      list_palettes: () => [saved],
      color_values: ({ hexes }) =>
        (hexes as string[]).map((h) => ({ hex: h, rgb: `rgb(of ${h})`, hsl: "", oklch: "" })),
      copy_to_clipboard: () => undefined,
    });
    render(<LibraryView />);
    const value = await screen.findByRole("button", { name: "Copy primary RGB rgb(of #87CEEB)" });
    await userEvent.click(value);
    expect(invoke).toHaveBeenCalledWith("copy_to_clipboard", { text: "rgb(of #87CEEB)" });
  });

  it("uses it in history and offers every format in details and settings", async () => {
    useSettings.setState({ valueFormat: "hsl" });
    const { unmount } = render(<HistoryView />);
    expect(await screen.findAllByText("hsl(197, 71%, 73%)")).not.toHaveLength(0);
    unmount();
    render(<ColorDetailsView />);
    for (const label of ["hex #87CEEB", "RGB rgb(135, 206, 235)", "HSL hsl(197, 71%, 73%)"]) {
      expect(await screen.findByRole("button", { name: `Copy ${label}` })).toBeInTheDocument();
    }
  });

  it("can be changed in settings", async () => {
    mockCommands({ list_history: () => [], update_settings: () => undefined });
    render(<SettingsView />);
    await userEvent.click(screen.getByRole("radio", { name: /OKLCH/ }));
    expect(useSettings.getState().valueFormat).toBe("oklch");
  });
});

describe("Copy and reset", () => {
  it("copies hex, RGB and OKLCH values when clicked", async () => {
    render(<ColorDetailsView />);
    await userEvent.click(
      await screen.findByRole("button", { name: "Copy RGB rgb(135, 206, 235)" }),
    );
    expect(invoke).toHaveBeenCalledWith("copy_to_clipboard", { text: "rgb(135, 206, 235)" });
    expect(await screen.findByText("Copied rgb(135, 206, 235)")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Copy hex #87CEEB" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Copy OKLCH oklch\(/ })).toBeInTheDocument();
  });

  it("resets settings only after confirming", async () => {
    mockCommands({
      reset_settings: () => ({ ...DEFAULT_SETTINGS, onboarded: true }),
      list_history: () => [],
    });
    useSettings.setState({ theme: "dark" });
    render(<SettingsView />);
    await userEvent.click(screen.getByRole("button", { name: "Reset settings to default" }));
    expect(useSettings.getState().theme).toBe("dark");
    await userEvent.click(screen.getByRole("button", { name: /Reset settings$/ }));
    expect(useSettings.getState()).toMatchObject({ theme: "system", onboarded: true });
  });
});
