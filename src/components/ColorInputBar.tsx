import { useEffect, useId, useRef, useState } from "react";
import { searchColorByName } from "../lib/color";
import { parseColorInput } from "../lib/parseColorInput";
import { startPicker } from "../lib/sampler";
import type { ColorName } from "../types/color";
import { Swatch } from "./Swatch";

/**
 * Hex, RGB or name (with autocomplete) plus "Pick from screen". Follows the ARIA combobox
 * pattern: arrows move through suggestions, Enter picks, Escape closes the list.
 */
export function ColorInputBar({
  onSelect,
  showPicker = true,
  label = "Colour",
  placeholder = "Type #87CEEB, rgb(135, 206, 235) or a name like sky blue",
}: {
  onSelect: (hex: string) => void;
  showPicker?: boolean;
  label?: string;
  placeholder?: string;
}) {
  const [text, setText] = useState("");
  const [suggestions, setSuggestions] = useState<ColorName[]>([]);
  const [active, setActive] = useState(-1);
  const [open, setOpen] = useState(false);
  const [message, setMessage] = useState<string | null>(null);
  const listId = useId();
  const inputId = useId();
  const requestId = useRef(0);

  useEffect(() => {
    const parsed = parseColorInput(text);
    if (parsed.kind !== "name") return;
    const id = ++requestId.current;
    const timer = setTimeout(() => {
      searchColorByName(parsed.query).then(
        (hits) => {
          if (id !== requestId.current) return;
          setSuggestions(hits.slice(0, 8));
          setActive(hits.length ? 0 : -1);
        },
        () => setSuggestions([]),
      );
    }, 120);
    return () => clearTimeout(timer);
  }, [text]);

  function choose(hex: string) {
    onSelect(hex);
    setText("");
    setSuggestions([]);
    setOpen(false);
    setMessage(null);
  }

  function submit() {
    const parsed = parseColorInput(text);
    if (parsed.kind === "color") choose(parsed.hex);
    else if (parsed.kind === "invalid") setMessage(parsed.message);
    else if (parsed.kind === "name") {
      const pick = suggestions[active] ?? suggestions[0];
      if (pick) choose(pick.hex);
      else setMessage(`No colour called “${parsed.query}”. Try another name or a hex value.`);
    }
  }

  const expanded = open && suggestions.length > 0;
  const optionId = (i: number) => `${listId}-opt-${i}`;

  return (
    <div className="relative flex flex-col gap-1">
      <div className="flex flex-wrap items-center gap-2">
        <label htmlFor={inputId} className="sr-only">
          {label}
        </label>
        <input
          id={inputId}
          type="text"
          role="combobox"
          aria-expanded={expanded}
          aria-controls={listId}
          aria-autocomplete="list"
          aria-activedescendant={expanded && active >= 0 ? optionId(active) : undefined}
          aria-describedby={message ? `${inputId}-msg` : undefined}
          autoComplete="off"
          spellCheck={false}
          value={text}
          placeholder={placeholder}
          onChange={(e) => {
            setText(e.target.value);
            if (parseColorInput(e.target.value).kind !== "name") setSuggestions([]);
            setOpen(true);
            setMessage(null);
          }}
          onBlur={() => setTimeout(() => setOpen(false), 150)}
          onKeyDown={(e) => {
            if (e.key === "ArrowDown" && suggestions.length) {
              e.preventDefault();
              setOpen(true);
              setActive((a) => (a + 1) % suggestions.length);
            } else if (e.key === "ArrowUp" && suggestions.length) {
              e.preventDefault();
              setActive((a) => (a - 1 + suggestions.length) % suggestions.length);
            } else if (e.key === "Enter") {
              e.preventDefault();
              submit();
            } else if (e.key === "Escape") {
              setOpen(false);
            }
          }}
          className="min-w-0 flex-1 rounded-md border border-line bg-surface px-3 py-2 text-base placeholder:text-muted"
        />
        <button
          type="button"
          onClick={submit}
          className="rounded-md border border-line bg-surface px-3 py-2 font-semibold hover:border-accent"
        >
          Show
        </button>
        {showPicker && (
          <button
            type="button"
            onClick={() => void startPicker()}
            className="rounded-md bg-accent px-3 py-2 font-semibold text-on-accent"
          >
            Pick from screen
          </button>
        )}
      </div>
      {message && (
        <p id={`${inputId}-msg`} role="alert" className="text-sm font-semibold text-bad">
          ⚠ {message}
        </p>
      )}
      <ul
        id={listId}
        role="listbox"
        aria-label="Matching colour names"
        hidden={!expanded}
        className="absolute top-full z-20 mt-1 max-h-80 w-full max-w-xl overflow-auto rounded-md border border-line bg-surface py-1 shadow-lg"
      >
        {suggestions.map((s, i) => (
          <li
            key={`${s.name}-${s.hex}`}
            id={optionId(i)}
            role="option"
            aria-selected={i === active}
            onMouseDown={(e) => {
              e.preventDefault();
              choose(s.hex);
            }}
            className={`flex cursor-pointer items-center gap-3 px-3 py-1.5 ${
              i === active ? "bg-raised font-semibold" : ""
            }`}
          >
            <Swatch hex={s.hex} />
            <span className="flex-1">{s.name}</span>{" "}
            <span className="font-mono text-sm text-muted">{s.hex}</span>
          </li>
        ))}
      </ul>
    </div>
  );
}
