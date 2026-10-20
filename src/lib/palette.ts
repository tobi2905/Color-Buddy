import type { ColorInfo, Palette, Role } from "../types/color";

export interface WorkingColor {
  hex: string;
  role: Role;
  info: ColorInfo;
  locked: boolean;
}

/** Keep locked colours in their slots, take the rest from a fresh palette. */
export function mergeRegenerated(current: WorkingColor[], fresh: Palette): WorkingColor[] {
  return fresh.colors.map((c, i) => {
    const old = current[i];
    return old && old.locked && old.role === c.role
      ? old
      : { hex: c.hex, role: c.role, info: c.info, locked: false };
  });
}
