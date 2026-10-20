/** What the colour input bar understood. */
export type ParsedInput =
  | { kind: "empty" }
  | { kind: "color"; hex: string; source: "hex" | "rgb" }
  | { kind: "name"; query: string }
  | { kind: "invalid"; message: string };

const HEX = /^#?([0-9a-f]{3}|[0-9a-f]{6})$/i;
// rgb(1, 2, 3), rgb(1 2 3 / 50%), rgba(1,2,3,0.5), or bare "1, 2, 3" / "1 2 3".
const RGB_FN =
  /^rgba?\(\s*(\d{1,3})\s*[,\s]\s*(\d{1,3})\s*[,\s]\s*(\d{1,3})\s*(?:[,/]\s*[\d.]+%?\s*)?\)$/i;
const RGB_BARE = /^(\d{1,3})\s*[,\s]\s*(\d{1,3})\s*[,\s]\s*(\d{1,3})$/;

const toHex = (n: number) => n.toString(16).padStart(2, "0").toUpperCase();

/**
 * Hex (`#87ceeb`, `87CEEB`, `#abc`), RGB (`rgb(135, 206, 235)`, `135 206 235`), otherwise a
 * name to search for. Hex without `#` needs a digit or six characters, so words like "bed"
 * are searched as names.
 */
export function parseColorInput(raw: string): ParsedInput {
  const text = raw.trim();
  if (!text) return { kind: "empty" };

  const hex = text.match(HEX);
  if (hex && (text.startsWith("#") || /\d/.test(text) || hex[1].length === 6)) {
    let h = hex[1].toUpperCase();
    if (h.length === 3) h = [...h].map((c) => c + c).join("");
    return { kind: "color", hex: `#${h}`, source: "hex" };
  }

  const rgb = text.match(RGB_FN) ?? text.match(RGB_BARE);
  if (rgb) {
    const values = rgb.slice(1, 4).map(Number);
    if (values.some((v) => v > 255)) {
      return { kind: "invalid", message: "RGB values go from 0 to 255." };
    }
    return { kind: "color", hex: `#${values.map(toHex).join("")}`, source: "rgb" };
  }

  if (text.startsWith("#")) {
    return { kind: "invalid", message: "A hex colour has 3 or 6 digits, like #87CEEB." };
  }
  return { kind: "name", query: text };
}
