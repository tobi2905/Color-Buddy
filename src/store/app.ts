import { create } from "zustand";

import type { SavedPalette } from "../lib/storage";

export type View = "color" | "palettes" | "saved" | "history" | "settings";

interface AppState {
  view: View;
  /** The colour shown in Colour details and used as the palette base. */
  currentHex: string;
  setView: (view: View) => void;
  /** Select a colour; jumps to Colour details unless told otherwise. */
  selectColor: (hex: string, view?: View) => void;
  /** A saved palette to open in the palette builder (consumed by the builder). */
  openedPalette: SavedPalette | null;
  openPalette: (palette: SavedPalette) => void;
  clearOpenedPalette: () => void;
}

export const useApp = create<AppState>()((set) => ({
  view: "color",
  currentHex: "#87CEEB",
  setView: (view) => set({ view }),
  selectColor: (hex, view = "color") => set({ currentHex: hex.toUpperCase(), view }),
  openedPalette: null,
  openPalette: (palette) =>
    set({ openedPalette: palette, currentHex: palette.baseHex.toUpperCase(), view: "palettes" }),
  clearOpenedPalette: () => set({ openedPalette: null }),
}));
