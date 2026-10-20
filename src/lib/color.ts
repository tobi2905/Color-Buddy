import { invoke } from "@tauri-apps/api/core";
import type {
  ColorDetails,
  ColorInfo,
  ColorName,
  ColorValues,
  PairCheck,
  Palette,
  Role,
  VisionProfile,
} from "../types/color";

export const getColorInfo = (hex: string) => invoke<ColorInfo>("get_color_info", { hex });
export const searchColorByName = (query: string) =>
  invoke<ColorName[]>("search_color_by_name", { query });
export const getColorDetails = (hex: string, profile: VisionProfile) =>
  invoke<ColorDetails>("get_color_details", { hex, profile });
export const generatePalettes = (baseHex: string) =>
  invoke<Palette[]>("generate_palettes", { baseHex });
export const checkPalette = (hexes: string[], roles?: Role[]) =>
  invoke<PairCheck[]>("check_palette", { hexes, roles: roles ?? null });
export const colorValues = (hexes: string[]) => invoke<ColorValues[]>("color_values", { hexes });
