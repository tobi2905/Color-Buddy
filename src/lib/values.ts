import { useEffect, useState } from "react";
import type { ValueFormat } from "../store/settings";
import type { ColorValues } from "../types/color";
import { colorValues } from "./color";

export const FORMAT_LABEL: Record<ValueFormat, string> = {
  hex: "hex",
  rgb: "RGB",
  hsl: "HSL",
  oklch: "OKLCH",
};

const cache = new Map<string, ColorValues>();

/**
 * Formatted values for colours known only by hex (saved palettes, name suggestions).
 * Hex is shown until the other formats arrive from the colour core.
 */
export function useColorValues(hexes: string[]): Record<string, ColorValues | undefined> {
  const key = hexes.join(",");
  const [, setVersion] = useState(0);
  useEffect(() => {
    const missing = [...new Set(key ? key.split(",") : [])].filter((h) => !cache.has(h));
    if (!missing.length) return;
    let current = true;
    colorValues(missing).then(
      (values) => {
        values.forEach((v, i) => cache.set(missing[i], v));
        if (current) setVersion((n) => n + 1);
      },
      () => {},
    );
    return () => {
      current = false;
    };
  }, [key]);
  return Object.fromEntries(hexes.map((h) => [h, cache.get(h)]));
}

/** The value in `format`, falling back to hex while it loads. */
export const valueIn = (hex: string, values: ColorValues | undefined, format: ValueFormat) =>
  values?.[format] ?? hex;
