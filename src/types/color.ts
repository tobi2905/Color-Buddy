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

/** One colour as text in every supported format. */
export interface ColorValues {
  hex: string;
  rgb: string;
  hsl: string;
  oklch: string;
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
  values: ColorValues;
}

export type VisionType = "normal" | "protan" | "deutan" | "tritan" | "achromat";

export interface VisionProfile {
  vision: VisionType;
  /** 0.0 – 1.0 */
  severity: number;
}

export type ReferenceMatch = "exact" | "similar" | "family";

export interface ReferenceSource {
  name: string;
  hex: string;
  match: ReferenceMatch;
  deltaE: number;
  /** e.g. "mint", "similar to mint", "greens in general" */
  label: string;
}

export interface ColorDetails {
  info: ColorInfo;
  /** Real-world objects. */
  references: string[];
  /** Moods first, then industries and uses. */
  associations: string[];
  /** Common pairings, as hex. */
  pairings: string[];
  /** Which reference entry these came from. */
  referenceSource: ReferenceSource;
  /** Named colours that look alike with the given profile, closest first. */
  confusedWith: ColorName[];
}

export type Scheme =
  "complementary" | "analogous" | "triadic" | "splitComplementary" | "monochrome";

export type Role = "background" | "surface" | "primary" | "accent" | "text";

export interface PaletteColor {
  hex: string;
  role: Role;
  info: ColorInfo;
}

export interface Palette {
  scheme: Scheme;
  baseHex: string;
  /** Background, surface, primary, accent(s), text — in that order. */
  colors: PaletteColor[];
  explanation: string;
}

export type PairVerdict = "distinct" | "caution" | "hardToTell";

export interface PairCheck {
  a: number;
  b: number;
  /** [vision type, ΔE2000 as seen with it, verdict]; deficiencies at full severity. */
  results: [VisionType, number, PairVerdict][];
  contrastRatio: number;
  /** Meant to look alike (background/surface): show, but never count as a warning. */
  subtle: boolean;
}
