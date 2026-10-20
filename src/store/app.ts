import { create } from "zustand";

export type View = "color" | "palettes" | "history" | "settings";

interface AppState {
  view: View;
  /** The colour shown in Colour details and used as the palette base. */
  currentHex: string;
  setView: (view: View) => void;
  /** Select a colour; jumps to Colour details unless told otherwise. */
  selectColor: (hex: string, view?: View) => void;
}

export const useApp = create<AppState>()((set) => ({
  view: "color",
  currentHex: "#87CEEB",
  setView: (view) => set({ view }),
  selectColor: (hex, view = "color") => set({ currentHex: hex.toUpperCase(), view }),
}));
