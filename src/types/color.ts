// Mirrors crates/color-core (ColorName, ColorInfo, Rgb).

export interface Rgb {
  r: number;
  g: number;
  b: number;
}

export interface ColorName {
  name: string;
  hex: string;
  /** CIEDE2000 distance from the looked-up colour (0 for search results). */
  deltaE: number;
}

export interface ColorInfo {
  hex: string;
  rgb: Rgb;
  /** OKLCH: L (0–1), C, h (degrees). */
  oklch: [number, number, number];
  /** The closest basic term. */
  basic: ColorName;
  /** Second-closest basic term when nearly as close (boundary colours), else null. */
  basicAlternative: ColorName | null;
  descriptive: ColorName;
  precise: ColorName;
  description: string;
}
