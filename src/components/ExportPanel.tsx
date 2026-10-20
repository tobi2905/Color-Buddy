import { useEffect, useId, useState } from "react";
import {
  copyToClipboard,
  exportPalette,
  exportPaletteToFile,
  type ExportFormat,
  type ExportPalette,
} from "../lib/storage";
import { useSettings } from "../store/settings";

const EXPORT_FORMATS: { value: ExportFormat; label: string; hint: string }[] = [
  { value: "text-detailed", label: "Detailed text", hint: "Role, name, hex and RGB per colour" },
  { value: "text-hex", label: "Hex values only", hint: "One per line" },
  { value: "text-names", label: "Names only", hint: "One per line" },
  { value: "text-names-hex", label: "Names with hex", hint: "One colour per line" },
  { value: "css", label: "CSS variables", hint: ":root { --color-primary: … }" },
  { value: "json", label: "JSON", hint: "Names, roles, values and check summary" },
  { value: "tailwind", label: "Tailwind config", hint: "theme.extend.colors snippet" },
];

/** Choose a format, preview it, then copy it or save it as a file. */
export function ExportPanel({ palette }: { palette: ExportPalette }) {
  const nameLevel = useSettings((s) => s.nameLevel);
  const [format, setFormat] = useState<ExportFormat>("text-detailed");
  const [text, setText] = useState("");
  const [status, setStatus] = useState<string | null>(null);
  const name = useId();
  const key = JSON.stringify(palette);

  useEffect(() => {
    let current = true;
    exportPalette(palette, format, nameLevel).then(
      (t) => current && setText(t),
      (err) => current && setText(`Couldn't export: ${String(err)}`),
    );
    return () => {
      current = false;
    };
    // `key` stands in for the palette object, which is rebuilt on every render.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [key, format, nameLevel]);

  async function run(action: () => Promise<string | null>) {
    try {
      setStatus(await action());
    } catch (err) {
      setStatus(`⚠ ${String(err)}`);
    }
  }

  return (
    <div className="flex flex-col gap-3 rounded-lg border border-line bg-surface p-4">
      <fieldset className="flex flex-col gap-2">
        <legend className="mb-1 font-semibold">Format</legend>
        <div className="grid gap-2 sm:grid-cols-2 lg:grid-cols-3">
          {EXPORT_FORMATS.map((f) => (
            <label
              key={f.value}
              className="grid cursor-pointer grid-cols-[auto_1fr] items-baseline gap-x-2 rounded-md border border-line p-2 has-[:checked]:border-accent has-[:checked]:bg-raised"
            >
              <input
                type="radio"
                name={name}
                checked={format === f.value}
                onChange={() => {
                  setFormat(f.value);
                  setStatus(null);
                }}
                className="accent-[var(--app-accent)]"
              />
              <span className="font-semibold">{f.label}</span>
              <span className="col-start-2 text-sm text-muted">{f.hint}</span>
            </label>
          ))}
        </div>
      </fieldset>
      <pre
        aria-label="Export preview"
        className="max-h-64 overflow-auto rounded-md bg-raised p-3 font-mono text-sm whitespace-pre"
      >
        {text}
      </pre>
      <div className="flex flex-wrap items-center gap-2">
        <button
          type="button"
          onClick={() =>
            void run(async () => {
              await copyToClipboard(text);
              return "✓ Copied to the clipboard";
            })
          }
          className="rounded-md bg-accent px-3 py-2 font-semibold text-on-accent"
        >
          Copy to clipboard
        </button>
        <button
          type="button"
          onClick={() =>
            void run(async () => {
              const path = await exportPaletteToFile(palette, format, nameLevel);
              return path ? `✓ Saved to ${path}` : null;
            })
          }
          className="rounded-md border border-line bg-surface px-3 py-2 font-semibold hover:border-accent"
        >
          Save as file…
        </button>
        <span role="status" className="font-semibold">
          {status}
        </span>
      </div>
    </div>
  );
}
