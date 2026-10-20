import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { DEFAULT_SETTINGS, useSettings } from "../../store/settings";
import { useApp } from "../../store/app";
import { checks, palette, skyBlueDetails } from "../../test/fixtures";
import { mockCommands } from "../../test/tauri";
import MainWindow from "./MainWindow";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/api/event", () => ({ listen: vi.fn(() => Promise.resolve(() => {})) }));

beforeEach(() => {
  useApp.setState({ currentHex: "#87CEEB", view: "color" });
  mockCommands({
    get_color_details: () => skyBlueDetails,
    get_color_info: () => skyBlueDetails.info,
    generate_palettes: () => [palette],
    check_palette: () => checks,
    list_history: () => [],
    get_screen_permission: () => true,
    get_sampler_status: () => new Promise(() => {}),
  });
});

describe("MainWindow", () => {
  it("starts with the welcome steps on first launch", async () => {
    useSettings.setState({ ...DEFAULT_SETTINGS });
    render(<MainWindow />);
    expect(screen.getByRole("heading", { name: "Welcome to Color-Buddy" })).toBeInTheDocument();
  });

  it("goes through onboarding to the colour view with the keyboard", async () => {
    useSettings.setState({ ...DEFAULT_SETTINGS });
    render(<MainWindow />);
    await userEvent.tab();
    await userEvent.keyboard("{Enter}"); // Next: vision
    expect(screen.getByRole("heading", { name: "Your colour vision" })).toBeInTheDocument();
    await userEvent.click(screen.getByRole("radio", { name: /Protan/ }));
    await userEvent.click(screen.getByRole("button", { name: "Next" }));
    expect(screen.getByRole("heading", { name: "Picker shortcut" })).toBeInTheDocument();
    // Skip through the remaining steps (macOS adds screen access).
    while (!screen.queryByRole("button", { name: "Start using Color-Buddy" })) {
      await userEvent.click(screen.getByRole("button", { name: "Next" }));
    }
    await userEvent.click(screen.getByRole("button", { name: "Start using Color-Buddy" }));
    expect(useSettings.getState()).toMatchObject({
      onboarded: true,
      profile: { vision: "protan" },
      profileUnsure: false,
    });
    expect(await screen.findByRole("heading", { level: 1, name: "sky blue" })).toBeInTheDocument();
  });

  it("navigates between screens and marks the current one", async () => {
    useSettings.setState({ ...DEFAULT_SETTINGS, onboarded: true });
    render(<MainWindow />);
    const nav = screen.getByRole("navigation", { name: "Main" });
    await userEvent.click(screen.getByRole("button", { name: "Palettes" }));
    expect(screen.getByRole("button", { name: "Palettes" })).toHaveAttribute(
      "aria-current",
      "page",
    );
    expect(await screen.findByRole("tab", { name: "Complementary" })).toHaveAttribute(
      "aria-selected",
      "true",
    );
    expect(await screen.findByRole("table")).toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: "Settings" }));
    expect(screen.getByRole("heading", { name: "Settings" })).toBeInTheDocument();
    expect(nav).toBeInTheDocument();
  });
});
