import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { ColorInfo } from "../types/color";
import type { HoveredEvent } from "../types/sampler";

export const freezePick = () => invoke<ColorInfo>("freeze_pick");
export const nudgeCursor = (dx: number, dy: number) => invoke<void>("nudge_cursor", { dx, dy });

export const onPickerFrozen = (cb: (e: HoveredEvent) => void): Promise<UnlistenFn> =>
  listen<HoveredEvent>("picker-frozen", (event) => cb(event.payload));

/** Payload of `picker-opened`: where the cursor was when the picker opened. */
export interface PickerOpened {
  /** Display under the cursor, and the cursor relative to it in CSS pixels. */
  display: number;
  localX: number;
  localY: number;
}

/** `null` when the cursor isn't on any display. */
export const onPickerOpened = (cb: (e: PickerOpened | null) => void): Promise<UnlistenFn> =>
  listen<PickerOpened | null>("picker-opened", (event) => cb(event.payload));

export const onPickerState = (
  name: "picker-resumed" | "picker-closed",
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
