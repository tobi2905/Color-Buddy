import type { NameLevel } from "../store/settings";
import type { ColorInfo, ColorName } from "../types/color";

/** The name the user chose to see first (Settings → name level). */
export const headlineName = (info: ColorInfo, level: NameLevel): ColorName => info[level];

/** Plain-language confidence from the CIEDE2000 distance to the named colour. */
export function matchLabel(deltaE: number): string | null {
  if (deltaE <= 1) return null;
  if (deltaE <= 5) return "close match";
  return "closest match";
}

export const VISION_LABEL = {
  normal: "Typical",
  protan: "Protan",
  deutan: "Deutan",
  tritan: "Tritan",
  achromat: "Achromat",
} as const;

export const ROLE_LABEL = {
  background: "Background",
  surface: "Surface",
  primary: "Primary",
  accent: "Accent",
  text: "Text",
} as const;
