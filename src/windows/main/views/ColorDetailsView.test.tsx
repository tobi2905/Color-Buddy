import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { useApp } from "../../../store/app";
import { DEFAULT_SETTINGS, useSettings } from "../../../store/settings";
import { info, skyBlueDetails } from "../../../test/fixtures";
import { mockCommands } from "../../../test/tauri";
import { ColorDetailsView } from "./ColorDetailsView";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/api/event", () => ({ listen: vi.fn(() => Promise.resolve(() => {})) }));

beforeEach(() => {
  useSettings.setState({ ...DEFAULT_SETTINGS, onboarded: true });
  useApp.setState({ currentHex: "#87CEEB", view: "color" });
  mockCommands({
    get_color_details: () => skyBlueDetails,
    get_color_info: ({ hex }) =>
      hex === "#FFFFFF" ? info("#FFFFFF", "white", "white") : info("#FFFF00", "yellow", "yellow"),
  });
});

describe("ColorDetailsView", () => {
  it("renders names, description, references, pairings and look-alikes", async () => {
    render(<ColorDetailsView />);
    expect(await screen.findByRole("heading", { level: 1, name: "sky blue" })).toBeInTheDocument();
    expect(screen.getByText("light, slightly desaturated, cool blue")).toBeInTheDocument();
    expect(screen.getByText(/rgb\(135, 206, 235\)/)).toBeInTheDocument();
    expect(screen.getByText("light blue")).toBeInTheDocument();
    expect(screen.getByText("clear daytime sky, bluebirds, summer days")).toBeInTheDocument();
    expect(await screen.findByRole("button", { name: /white #FFFFFF/ })).toBeInTheDocument();
    expect(screen.getByText(/With moderate deutan, these named colours/)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /light grey #D3D3D3/ })).toBeInTheDocument();
  });

  it("uses the chosen name level as the headline", async () => {
    useSettings.setState({ nameLevel: "basic" });
    render(<ColorDetailsView />);
    expect(await screen.findByRole("heading", { level: 1, name: /^blue/ })).toHaveTextContent(
      "closest match",
    );
  });

  it("explains the empty look-alike list for typical colour vision", async () => {
    useSettings.setState({ profile: { vision: "normal", severity: 0 } });
    mockCommands({
      get_color_details: () => ({ ...skyBlueDetails, confusedWith: [] }),
      get_color_info: () => info("#FFFFFF", "white", "white"),
    });
    render(<ColorDetailsView />);
    expect(await screen.findByText(/no look-alike warnings/)).toBeInTheDocument();
  });

  it("opens a pairing or look-alike when clicked", async () => {
    render(<ColorDetailsView />);
    await userEvent.click(await screen.findByRole("button", { name: /light grey #D3D3D3/ }));
    expect(useApp.getState().currentHex).toBe("#D3D3D3");
  });
});
