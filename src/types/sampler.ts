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

/** Temporary Phase 1 payload of the `color-hovered` event. */
export interface HoveredEvent {
  hex: string;
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
