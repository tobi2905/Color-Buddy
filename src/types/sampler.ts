import type { ColorInfo, Rgb } from "./color";

// Mirrors src-tauri/src/sampler.rs and commands.rs (Phase 1 debug types).

export interface DisplayInfo {
  id: number;
  scale: number;
  profile: string;
  isSrgb: boolean;
  convertedToSrgb: boolean;
}

export interface SamplerStatus {
  running: boolean;
  size: number;
  samplesPerSecond: number;
  avgCaptureMs: number;
  firstSampleMs: number | null;
  display: DisplayInfo | null;
  error: string | null;
  errorKind: "permission" | "capture" | null;
}

/** Payload of the `color-hovered` and `picker-frozen` events. */
export interface HoveredEvent {
  /** Cursor in platform-native global coordinates. */
  x: number;
  y: number;
  /** Display under the cursor, and the cursor relative to it in CSS pixels. */
  display: number;
  localX: number;
  localY: number;
  info: ColorInfo;
  /** The N×N sample, row by row; the centre is the pixel under the cursor. */
  loupe: Rgb[][];
}

export interface Patch {
  hex: string;
  /** Centre in CSS pixels relative to the webview's top-left corner. */
  x: number;
  y: number;
}

export interface PatchResult {
  expected: string;
  size: number;
  center: string;
  allMatch: boolean;
  maxChannelDiff: number;
}
