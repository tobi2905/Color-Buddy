import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { HoveredEvent, Patch, PatchResult, SamplerStatus } from "../types/sampler";

export const startPicker = () => invoke<void>("start_picker");
export const stopPicker = () => invoke<void>("stop_picker");
export const setSampleSize = (size: number) => invoke<void>("set_sample_size", { size });
export const getSamplerStatus = () => invoke<SamplerStatus>("get_sampler_status");
export const openScreenRecordingSettings = () => invoke<void>("open_screen_recording_settings");
export const selfTest = (patches: Patch[]) => invoke<PatchResult[]>("self_test", { patches });

export const onColorHovered = (cb: (e: HoveredEvent) => void): Promise<UnlistenFn> =>
  listen<HoveredEvent>("color-hovered", (event) => cb(event.payload));

export const onSamplerStatus = (cb: (s: SamplerStatus) => void): Promise<UnlistenFn> =>
  listen<SamplerStatus>("sampler-status", (event) => cb(event.payload));
