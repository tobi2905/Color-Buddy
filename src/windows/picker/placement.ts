/** Gap between the cursor and the panel, in CSS pixels. */
export const OFFSET = 24;

/**
 * Top-left of the panel: below-right of the cursor, flipped left/up when it would leave the
 * display. Never covers the cursor itself.
 */
export function placePanel(
  cursor: { x: number; y: number },
  panel: { width: number; height: number },
  viewport: { width: number; height: number },
): { left: number; top: number } {
  let left = cursor.x + OFFSET;
  if (left + panel.width > viewport.width) left = cursor.x - OFFSET - panel.width;
  let top = cursor.y + OFFSET;
  if (top + panel.height > viewport.height) top = cursor.y - OFFSET - panel.height;
  return { left: Math.max(0, left), top: Math.max(0, top) };
}

export { matchLabel } from "../../lib/names";
