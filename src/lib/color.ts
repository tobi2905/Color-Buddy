import { invoke } from "@tauri-apps/api/core";
import type { ColorInfo, ColorName } from "../types/color";

export const getColorInfo = (hex: string) => invoke<ColorInfo>("get_color_info", { hex });
export const searchColorByName = (query: string) =>
  invoke<ColorName[]>("search_color_by_name", { query });
