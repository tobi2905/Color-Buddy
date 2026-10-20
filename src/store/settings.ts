import { create } from "zustand";
import { getSettings, resetSettings, updateSettings } from "../lib/storage";
import type { VisionProfile, VisionType } from "../types/color";

export type NameLevel = "basic" | "descriptive" | "precise";
export type Theme = "system" | "light" | "dark";
export type SampleSize = 1 | 3 | 5;
export type ValueFormat = "hex" | "rgb" | "hsl" | "oklch";

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
  /** How colour values are shown and copied in lists. */
  valueFormat: ValueFormat;
}

export const DEFAULT_SETTINGS: Settings = {
  onboarded: false,
  profile: { vision: "deutan", severity: 0.6 },
  profileUnsure: true,
  hotkey: "CommandOrControl+Shift+C",
  sampleSize: 5,
  theme: "system",
  nameLevel: "descriptive",
  valueFormat: "hex",
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
  /** Change settings; saved to the app database. */
  update: (patch: Partial<Settings>) => void;
  /** Back to defaults (the welcome steps stay done). */
  reset: () => Promise<void>;
}

const pick = (s: Settings): Settings => ({
  onboarded: s.onboarded,
  profile: s.profile,
  profileUnsure: s.profileUnsure,
  hotkey: s.hotkey,
  sampleSize: s.sampleSize,
  theme: s.theme,
  nameLevel: s.nameLevel,
  valueFormat: s.valueFormat,
});

export const useSettings = create<SettingsStore>()((set, get) => ({
  ...DEFAULT_SETTINGS,
  update: (patch) => {
    set(patch);
    updateSettings(pick(get())).catch((err) => console.error("saving settings:", err));
  },
  reset: async () => {
    set(await resetSettings());
  },
}));

/** Key used by versions that kept settings in local storage (before Phase 7). */
export const LEGACY_STORAGE_KEY = "color-buddy-settings";

/**
 * Load settings from the app database before the first render. The first time, settings an
 * older version kept in local storage are imported, so nothing resets after the update.
 */
export async function loadSettings(): Promise<void> {
  try {
    const stored = await getSettings();
    let settings = stored.settings;
    if (!stored.saved) {
      const legacy = readLegacy();
      if (legacy) {
        settings = { ...settings, ...legacy };
        await updateSettings(settings);
      }
    }
    useSettings.setState(settings);
  } catch (err) {
    console.error("loading settings:", err);
  }
}

function readLegacy(): Partial<Settings> | null {
  try {
    const raw = localStorage.getItem(LEGACY_STORAGE_KEY);
    if (!raw) return null;
    const state = JSON.parse(raw)?.state;
    localStorage.removeItem(LEGACY_STORAGE_KEY);
    return state && typeof state === "object" ? pick({ ...DEFAULT_SETTINGS, ...state }) : null;
  } catch {
    return null;
  }
}

export function profileLabel(profile: VisionProfile): string {
  if (profile.vision === "normal") return "typical colour vision";
  const type = VISION_OPTIONS.find((o) => o.value === profile.vision)?.label ?? profile.vision;
  const severity = SEVERITY_OPTIONS.reduce((best, o) =>
    Math.abs(o.value - profile.severity) < Math.abs(best.value - profile.severity) ? o : best,
  ).label;
  return `${severity.toLowerCase()} ${type.toLowerCase()}`;
}
