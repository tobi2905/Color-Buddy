import { create } from "zustand";
import { createJSONStorage, persist } from "zustand/middleware";
import type { VisionProfile, VisionType } from "../types/color";

export type NameLevel = "basic" | "descriptive" | "precise";
export type Theme = "system" | "light" | "dark";
export type SampleSize = 1 | 3 | 5;

export interface Settings {
  onboarded: boolean;
  profile: VisionProfile;
  /** Chose "not sure" during onboarding (profile then defaults to moderate deutan). */
  profileUnsure: boolean;
  /** Accelerator for the picker, e.g. "CommandOrControl+Shift+C". */
  hotkey: string;
  sampleSize: SampleSize;
  theme: Theme;
  nameLevel: NameLevel;
}

export const DEFAULT_SETTINGS: Settings = {
  onboarded: false,
  profile: { vision: "deutan", severity: 0.6 },
  profileUnsure: true,
  hotkey: "CommandOrControl+Shift+C",
  sampleSize: 5,
  theme: "system",
  nameLevel: "descriptive",
};

/** The "not sure" default: moderate deutan, the most common type. */
export const UNSURE_PROFILE: VisionProfile = { vision: "deutan", severity: 0.6 };

export const VISION_OPTIONS: { value: VisionType; label: string; hint: string }[] = [
  {
    value: "normal",
    label: "Typical colour vision",
    hint: "No colour vision deficiency. Checks still cover colourblind viewers of your work.",
  },
  { value: "deutan", label: "Deutan", hint: "Green-weak. The most common type." },
  { value: "protan", label: "Protan", hint: "Red-weak. Reds look darker." },
  { value: "tritan", label: "Tritan", hint: "Blue-yellow. Rare." },
  { value: "achromat", label: "Achromat", hint: "Little or no colour." },
];

export const SEVERITY_OPTIONS: { value: number; label: string }[] = [
  { value: 0.3, label: "Mild" },
  { value: 0.6, label: "Moderate" },
  { value: 0.8, label: "Strong" },
  { value: 1.0, label: "Complete" },
];

interface SettingsStore extends Settings {
  update: (patch: Partial<Settings>) => void;
}

/** Kept in local storage until Phase 7 moves settings into the app database. */
export const useSettings = create<SettingsStore>()(
  persist(
    (set) => ({
      ...DEFAULT_SETTINGS,
      update: (patch) => set(patch),
    }),
    {
      name: "color-buddy-settings",
      version: 1,
      storage: createJSONStorage(() => localStorage),
      partialize: ({ update: _update, ...settings }) => settings,
    },
  ),
);

export function profileLabel(profile: VisionProfile): string {
  if (profile.vision === "normal") return "typical colour vision";
  const type = VISION_OPTIONS.find((o) => o.value === profile.vision)?.label ?? profile.vision;
  const severity = SEVERITY_OPTIONS.reduce((best, o) =>
    Math.abs(o.value - profile.severity) < Math.abs(best.value - profile.severity) ? o : best,
  ).label;
  return `${severity.toLowerCase()} ${type.toLowerCase()}`;
}
