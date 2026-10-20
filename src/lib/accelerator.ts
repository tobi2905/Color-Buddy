/** Global-shortcut strings ("CommandOrControl+Shift+C") ↔ key presses and display text. */

export const isMac = typeof navigator !== "undefined" && /Mac/.test(navigator.userAgent);

const MODIFIER_CODES = new Set([
  "ShiftLeft",
  "ShiftRight",
  "ControlLeft",
  "ControlRight",
  "AltLeft",
  "AltRight",
  "MetaLeft",
  "MetaRight",
]);

/** The key part of an accelerator from a `KeyboardEvent.code`, or null for unsupported keys. */
function keyName(code: string): string | null {
  if (/^Key[A-Z]$/.test(code)) return code.slice(3);
  if (/^Digit[0-9]$/.test(code)) return code.slice(5);
  if (/^F([1-9]|1[0-9]|2[0-4])$/.test(code)) return code;
  const named: Record<string, string> = {
    Space: "Space",
    Minus: "-",
    Equal: "=",
    BracketLeft: "[",
    BracketRight: "]",
    Semicolon: ";",
    Quote: "'",
    Comma: ",",
    Period: ".",
    Slash: "/",
    Backslash: "\\",
    Backquote: "`",
  };
  return named[code] ?? null;
}

export type RecordResult =
  | { kind: "pending" }
  | { kind: "accelerator"; accelerator: string }
  | { kind: "invalid"; message: string };

/** Turn a key press into an accelerator. Modifier-only presses are "pending". */
export function acceleratorFromEvent(e: {
  code: string;
  metaKey: boolean;
  ctrlKey: boolean;
  altKey: boolean;
  shiftKey: boolean;
}): RecordResult {
  if (MODIFIER_CODES.has(e.code)) return { kind: "pending" };
  const key = keyName(e.code);
  if (!key) return { kind: "invalid", message: "That key can't be used in a shortcut." };
  if (!e.metaKey && !e.ctrlKey && !e.altKey) {
    return {
      kind: "invalid",
      message: `Hold ${isMac ? "⌘, ⌃ or ⌥" : "Ctrl or Alt"} too, so the shortcut doesn't block typing.`,
    };
  }
  const parts: string[] = [];
  if (e.metaKey) parts.push(isMac ? "Command" : "Super");
  if (e.ctrlKey) parts.push("Control");
  if (e.altKey) parts.push("Alt");
  if (e.shiftKey) parts.push("Shift");
  parts.push(key);
  return { kind: "accelerator", accelerator: parts.join("+") };
}

/** Human-readable shortcut: "⌘⇧C" on macOS, "Ctrl + Shift + C" elsewhere. */
export function formatAccelerator(accelerator: string, mac = isMac): string {
  const parts = accelerator.split("+");
  const key = parts.pop() ?? "";
  const mods = parts.map((p) => p.toLowerCase());
  const has = (...names: string[]) => mods.some((m) => names.includes(m));
  const primary = has("commandorcontrol", "cmdorctrl");
  if (mac) {
    return (
      (has("control", "ctrl") ? "⌃" : "") +
      (has("alt", "option") ? "⌥" : "") +
      (has("shift") ? "⇧" : "") +
      (primary || has("command", "cmd", "super", "meta") ? "⌘" : "") +
      key
    );
  }
  const out: string[] = [];
  if (primary || has("control", "ctrl")) out.push("Ctrl");
  if (has("super", "meta", "command", "cmd")) out.push("Win");
  if (has("alt", "option")) out.push("Alt");
  if (has("shift")) out.push("Shift");
  out.push(key);
  return out.join(" + ");
}
