import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { mockCommands } from "../test/tauri";
import { DEFAULT_SETTINGS, LEGACY_STORAGE_KEY, loadSettings, useSettings } from "./settings";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/api/event", () => ({ listen: vi.fn(() => Promise.resolve(() => {})) }));

beforeEach(() => {
  localStorage.clear();
  useSettings.setState({ ...DEFAULT_SETTINGS });
  vi.mocked(invoke).mockReset();
});

describe("settings store", () => {
  it("loads saved settings from the app database", async () => {
    mockCommands({
      get_settings: () => ({ saved: true, settings: { ...DEFAULT_SETTINGS, theme: "dark" } }),
    });
    await loadSettings();
    expect(useSettings.getState().theme).toBe("dark");
  });

  it("imports settings an older version kept in local storage, once", async () => {
    localStorage.setItem(
      LEGACY_STORAGE_KEY,
      JSON.stringify({
        state: { onboarded: true, hotkey: "Alt+P", nameLevel: "basic" },
        version: 1,
      }),
    );
    const saved: unknown[] = [];
    mockCommands({
      get_settings: () => ({ saved: false, settings: DEFAULT_SETTINGS }),
      update_settings: ({ settings }) => void saved.push(settings),
    });
    await loadSettings();
    expect(useSettings.getState()).toMatchObject({
      onboarded: true,
      hotkey: "Alt+P",
      nameLevel: "basic",
    });
    expect(saved).toHaveLength(1);
    expect(localStorage.getItem(LEGACY_STORAGE_KEY)).toBeNull();
  });

  it("saves every change and resets to defaults", async () => {
    const saved: Record<string, unknown>[] = [];
    mockCommands({
      update_settings: ({ settings }) => void saved.push(settings as Record<string, unknown>),
      reset_settings: () => ({ ...DEFAULT_SETTINGS, onboarded: true }),
    });
    useSettings.getState().update({ theme: "light" });
    expect(saved[saved.length - 1]).toMatchObject({ theme: "light" });
    expect(saved[saved.length - 1]).not.toHaveProperty("update");
    await useSettings.getState().reset();
    expect(useSettings.getState()).toMatchObject({ theme: "system", onboarded: true });
  });
});
