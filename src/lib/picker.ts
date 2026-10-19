import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { ColorInfo } from "../types/color";
import type { HoveredEvent } from "../types/sampler";

export const freezePick = () => invoke<ColorInfo>("freeze_pick");
export const nudgeCursor = (dx: number, dy: number) => invoke<void>("nudge_cursor", { dx, dy });

export const onPickerFrozen = (cb: (e: HoveredEvent) => void): Promise<UnlistenFn> =>
  listen<HoveredEvent>("picker-frozen", (event) => cb(event.payload));

/** "picker-opened" | "picker-resumed" | "picker-closed" */
export const onPickerState = (
  name: "picker-opened" | "picker-resumed" | "picker-closed",
  cb: () => void,
): Promise<UnlistenFn> => listen(name, () => cb());

export interface HistoryEntry {
  id: number;
  hex: string;
  /** Unix time in milliseconds. */
  pickedAt: number;
  source: string;
}

/** Newest first. */
export const listHistory = (limit?: number) =>
  invoke<HistoryEntry[]>("list_history", { limit: limit ?? null });
