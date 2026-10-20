import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { Settings } from "../store/settings";
import type { Role, Scheme } from "../types/color";

export interface StoredSettings {
  settings: Settings;
  /** False until settings were saved once (older versions kept them in local storage). */
  saved: boolean;
}

export const getSettings = () => invoke<StoredSettings>("get_settings");
export const updateSettings = (settings: Settings) => invoke<void>("update_settings", { settings });
export const resetSettings = () => invoke<Settings>("reset_settings");
export const onSettingsChanged = (cb: (s: Settings) => void): Promise<UnlistenFn> =>
  listen<Settings>("settings-changed", (e) => cb(e.payload));

export const deleteHistoryEntry = (id: number) => invoke<void>("delete_history_entry", { id });
export const clearHistory = () => invoke<void>("clear_history");

export interface PaletteColorInput {
  hex: string;
  role: Role;
  locked: boolean;
}

export interface SavedPalette {
  id: number;
  name: string;
  /** Unix time in milliseconds. */
  createdAt: number;
  baseHex: string;
  scheme: Scheme;
  colors: PaletteColorInput[];
}

export const savePalette = (palette: Omit<SavedPalette, "id" | "createdAt"> & { id?: number }) =>
  invoke<SavedPalette>("save_palette", { palette: { id: null, ...palette } });
export const listPalettes = () => invoke<SavedPalette[]>("list_palettes");
export const renamePalette = (id: number, name: string) =>
  invoke<void>("rename_palette", { id, name });
export const deletePalette = (id: number) => invoke<void>("delete_palette", { id });

export type ExportFormat =
  "css" | "json" | "tailwind" | "text-detailed" | "text-hex" | "text-names" | "text-names-hex";

export interface ExportPalette {
  name: string;
  scheme: Scheme;
  baseHex: string;
  colors: { hex: string; role: Role }[];
}

export const exportPalette = (
  palette: ExportPalette,
  format: ExportFormat,
  nameLevel: Settings["nameLevel"],
) => invoke<string>("export_palette", { palette, format, nameLevel });

/** Returns the saved path, or null when the save dialog was cancelled. */
export const exportPaletteToFile = (
  palette: ExportPalette,
  format: ExportFormat,
  nameLevel: Settings["nameLevel"],
) => invoke<string | null>("export_palette_to_file", { palette, format, nameLevel });

export const copyToClipboard = (text: string) => invoke<void>("copy_to_clipboard", { text });
