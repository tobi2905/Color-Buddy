import { useEffect, useState } from "react";
import { ConfirmButton } from "../../../components/ConfirmButton";
import { CopyValue } from "../../../components/CopyValue";
import { ExportPanel } from "../../../components/ExportPanel";
import { Swatch } from "../../../components/Swatch";
import { ROLE_LABEL } from "../../../lib/names";
import {
  deletePalette,
  listPalettes,
  renamePalette,
  type SavedPalette,
} from "../../../lib/storage";
import { useApp } from "../../../store/app";

const SCHEME_LABEL = {
  complementary: "Complementary",
  analogous: "Analogous",
  triadic: "Triadic",
  splitComplementary: "Split-complementary",
  monochrome: "Monochrome",
} as const;

const dateFormat = new Intl.DateTimeFormat(undefined, { dateStyle: "medium" });

/** Saved palettes: open, rename, export or delete. */
export function LibraryView() {
  const [palettes, setPalettes] = useState<SavedPalette[] | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let current = true;
    listPalettes().then(
      (list) => current && setPalettes(list),
      (err) => current && setError(String(err)),
    );
    return () => {
      current = false;
    };
  }, []);

  return (
    <div className="flex max-w-4xl flex-col gap-4">
      <header className="flex flex-col gap-1">
        <h1 className="text-3xl font-extrabold">Saved palettes</h1>
        <p className="text-muted">Newest first. Open one to keep editing or checking it.</p>
      </header>
      {error && (
        <p role="alert" className="font-semibold text-bad">
          ⚠ {error}
        </p>
      )}
      {palettes === null ? (
        !error && <p className="text-muted">Loading…</p>
      ) : palettes.length === 0 ? (
        <p className="text-muted">
          No saved palettes yet. Build one under Palettes and choose “Save palette”.
        </p>
      ) : (
        <ul className="flex flex-col gap-3">
          {palettes.map((p) => (
            <PaletteCard
              key={p.id}
              palette={p}
              onChanged={(next) =>
                setPalettes((list) => list?.map((x) => (x.id === next.id ? next : x)) ?? null)
              }
              onDeleted={() => setPalettes((list) => list?.filter((x) => x.id !== p.id) ?? null)}
            />
          ))}
        </ul>
      )}
    </div>
  );
}

function PaletteCard({
  palette: p,
  onChanged,
  onDeleted,
}: {
  palette: SavedPalette;
  onChanged: (p: SavedPalette) => void;
  onDeleted: () => void;
}) {
  const openPalette = useApp((s) => s.openPalette);
  const [renaming, setRenaming] = useState(false);
  const [name, setName] = useState(p.name);
  const [exporting, setExporting] = useState(false);
  const [message, setMessage] = useState<string | null>(null);
  const headingId = `palette-${p.id}`;

  async function rename() {
    try {
      await renamePalette(p.id, name);
      onChanged({ ...p, name: name.trim() });
      setRenaming(false);
      setMessage(null);
    } catch (err) {
      setMessage(`⚠ ${String(err)}`);
    }
  }

  return (
    <li
      aria-labelledby={headingId}
      className="flex flex-col gap-3 rounded-lg border border-line bg-surface p-4"
    >
      <div className="flex flex-wrap items-baseline justify-between gap-2">
        {renaming ? (
          <form
            className="flex flex-wrap items-center gap-2"
            onSubmit={(e) => {
              e.preventDefault();
              void rename();
            }}
          >
            <label htmlFor={`${headingId}-name`} className="sr-only">
              New name for {p.name}
            </label>
            <input
              id={`${headingId}-name`}
              value={name}
              onChange={(e) => setName(e.target.value)}
              className="rounded-md border border-line bg-app px-3 py-1.5"
            />
            <button
              type="submit"
              className="rounded-md bg-accent px-3 py-1.5 font-semibold text-on-accent"
            >
              Save name
            </button>
            <button
              type="button"
              onClick={() => {
                setRenaming(false);
                setName(p.name);
                setMessage(null);
              }}
              className="rounded-md border border-line px-3 py-1.5 font-semibold"
            >
              Cancel
            </button>
          </form>
        ) : (
          <h2 id={headingId} className="text-xl font-extrabold">
            {p.name}
          </h2>
        )}
        <p className="text-sm text-muted">
          {SCHEME_LABEL[p.scheme]} · {p.colors.length} colours · saved{" "}
          {dateFormat.format(new Date(p.createdAt))}
        </p>
      </div>

      <div className="flex h-10 overflow-hidden rounded-md border border-line" aria-hidden="true">
        {p.colors.map((c, i) => (
          <Swatch key={i} hex={c.hex} className="h-full flex-1 rounded-none border-0" />
        ))}
      </div>
      <ul className="flex flex-wrap gap-x-4 gap-y-1 text-sm text-muted">
        {p.colors.map((c, i) => (
          <li key={i} className="inline-flex items-baseline gap-1">
            {ROLE_LABEL[c.role]}{" "}
            <CopyValue label={`${ROLE_LABEL[c.role].toLowerCase()} hex`} value={c.hex} compact />
          </li>
        ))}
      </ul>

      <div className="flex flex-wrap items-center gap-2">
        <button
          type="button"
          onClick={() => openPalette(p)}
          className="rounded-md bg-accent px-3 py-1.5 font-semibold text-on-accent"
        >
          Open
        </button>
        <button
          type="button"
          onClick={() => setRenaming(true)}
          className="rounded-md border border-line bg-surface px-3 py-1.5 font-semibold hover:border-accent"
        >
          Rename
        </button>
        <button
          type="button"
          aria-expanded={exporting}
          onClick={() => setExporting(!exporting)}
          className="rounded-md border border-line bg-surface px-3 py-1.5 font-semibold hover:border-accent"
        >
          Export or copy
        </button>
        <ConfirmButton
          question={`Delete “${p.name}”?`}
          confirmLabel="Delete palette"
          onConfirm={() =>
            deletePalette(p.id).then(onDeleted, (err) => setMessage(`⚠ ${String(err)}`))
          }
        >
          Delete
        </ConfirmButton>
        {message && (
          <span role="alert" className="font-semibold text-bad">
            {message}
          </span>
        )}
      </div>

      {exporting && (
        <ExportPanel
          palette={{
            name: p.name,
            scheme: p.scheme,
            baseHex: p.baseHex,
            colors: p.colors.map(({ hex, role }) => ({ hex, role })),
          }}
        />
      )}
    </li>
  );
}
