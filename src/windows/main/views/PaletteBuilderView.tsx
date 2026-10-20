import { useEffect, useMemo, useState } from "react";
import { CheckGrid } from "../../../components/CheckGrid";
import { ColorInputBar } from "../../../components/ColorInputBar";
import { ExportPanel } from "../../../components/ExportPanel";
import { Swatch } from "../../../components/Swatch";
import { Tabs } from "../../../components/Tabs";
import { checkPalette, generatePalettes, getColorInfo } from "../../../lib/color";
import { headlineName, ROLE_LABEL } from "../../../lib/names";
import { mergeRegenerated, type WorkingColor } from "../../../lib/palette";
import { savePalette } from "../../../lib/storage";
import { useApp } from "../../../store/app";
import { useSettings } from "../../../store/settings";
import type { PairCheck, Scheme } from "../../../types/color";

const SCHEMES: { id: Scheme; label: string }[] = [
  { id: "complementary", label: "Complementary" },
  { id: "analogous", label: "Analogous" },
  { id: "triadic", label: "Triadic" },
  { id: "splitComplementary", label: "Split-complementary" },
  { id: "monochrome", label: "Monochrome" },
];

/** Scheme tabs, editable swatches (replace / lock / regenerate) and the check grid. */
export function PaletteBuilderView() {
  const base = useApp((s) => s.currentHex);
  const openedPalette = useApp((s) => s.openedPalette);
  const clearOpenedPalette = useApp((s) => s.clearOpenedPalette);
  const nameLevel = useSettings((s) => s.nameLevel);
  const vision = useSettings((s) => s.profile.vision);
  const [scheme, setScheme] = useState<Scheme>("complementary");
  const [working, setWorking] = useState<Partial<Record<Scheme, WorkingColor[]>>>({});
  const [explanations, setExplanations] = useState<Partial<Record<Scheme, string>>>({});
  const [checks, setChecks] = useState<PairCheck[]>([]);
  const [editing, setEditing] = useState<number | null>(null);
  const [error, setError] = useState<string | null>(null);
  /** The base colour the current schemes were generated from. */
  const [loadedBase, setLoadedBase] = useState<string | null>(null);
  /** The saved palette being edited, if any. */
  const [saved, setSaved] = useState<{ id: number; name: string } | null>(null);
  const [name, setName] = useState("");
  const [saveStatus, setSaveStatus] = useState<string | null>(null);

  // A new base colour starts every scheme afresh.
  useEffect(() => {
    let current = true;
    generatePalettes(base).then(
      (palettes) => {
        if (!current) return;
        setWorking(Object.fromEntries(palettes.map((p) => [p.scheme, mergeRegenerated([], p)])));
        setExplanations(Object.fromEntries(palettes.map((p) => [p.scheme, p.explanation])));
        setSaved(null);
        setName("");
        setSaveStatus(null);
        setError(null);
        setLoadedBase(base);
      },
      (err) => current && setError(String(err)),
    );
    return () => {
      current = false;
    };
  }, [base]);

  // Open a saved palette once its base colour's schemes are ready.
  useEffect(() => {
    if (!openedPalette || loadedBase !== openedPalette.baseHex) return;
    let current = true;
    Promise.all(openedPalette.colors.map((c) => getColorInfo(c.hex))).then(
      (infos) => {
        if (!current) return;
        const restored = openedPalette.colors.map((c, i) => ({ ...c, info: infos[i] }));
        setWorking((w) => ({ ...w, [openedPalette.scheme]: restored }));
        setScheme(openedPalette.scheme);
        setSaved({ id: openedPalette.id, name: openedPalette.name });
        setName(openedPalette.name);
        setSaveStatus(null);
        clearOpenedPalette();
      },
      (err) => current && setError(String(err)),
    );
    return () => {
      current = false;
    };
  }, [openedPalette, loadedBase, clearOpenedPalette]);

  const colors = useMemo(() => working[scheme] ?? [], [working, scheme]);
  const primaryName = colors.find((c) => c.role === "primary")?.info.descriptive.name ?? base;
  const schemeLabel = SCHEMES.find((x) => x.id === scheme)?.label ?? scheme;
  const paletteName = name.trim() || `${primaryName} ${schemeLabel.toLowerCase()}`;
  const hexKey = colors.map((c) => c.hex).join(",");

  useEffect(() => {
    if (!colors.length) return;
    let current = true;
    checkPalette(
      colors.map((c) => c.hex),
      colors.map((c) => c.role),
    ).then(
      (result) => current && setChecks(result),
      (err) => current && setError(String(err)),
    );
    return () => {
      current = false;
    };
    // Re-check only when the colours change, not on lock toggles.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [hexKey]);

  const update = (next: WorkingColor[]) => setWorking((w) => ({ ...w, [scheme]: next }));

  async function replace(index: number, hex: string) {
    const info = await getColorInfo(hex);
    update(colors.map((c, i) => (i === index ? { ...c, hex: info.hex, info, locked: true } : c)));
    setEditing(null);
  }

  async function save(asNew: boolean) {
    try {
      const result = await savePalette({
        id: asNew ? undefined : saved?.id,
        name: paletteName,
        baseHex: base,
        scheme,
        colors: colors.map(({ hex, role, locked }) => ({ hex, role, locked })),
      });
      setSaved({ id: result.id, name: result.name });
      setName(result.name);
      setSaveStatus(`✓ Saved “${result.name}”`);
    } catch (err) {
      setSaveStatus(`⚠ ${String(err)}`);
    }
  }

  async function regenerate() {
    const primary = colors.find((c) => c.role === "primary")?.hex ?? base;
    const fresh = (await generatePalettes(primary)).find((p) => p.scheme === scheme);
    if (!fresh) return;
    update(mergeRegenerated(colors, fresh));
    setExplanations((e) => ({ ...e, [scheme]: fresh.explanation }));
  }

  return (
    <div className="flex max-w-6xl flex-col gap-6">
      <header className="flex flex-col gap-1">
        <h1 className="text-3xl font-extrabold">Palettes</h1>
        <p className="text-muted">
          Built from <span className="font-mono">{base}</span>. Lock colours you want to keep,
          replace any colour, then regenerate the rest.
        </p>
      </header>

      {error && (
        <p role="alert" className="font-semibold text-bad">
          ⚠ {error}
        </p>
      )}

      <Tabs
        tabs={SCHEMES}
        selected={scheme}
        onSelect={(s) => {
          setScheme(s);
          setEditing(null);
        }}
        label="Palette scheme"
        idPrefix="scheme"
      />

      <section
        id="scheme-panel"
        role="tabpanel"
        aria-labelledby={`scheme-tab-${scheme}`}
        className="flex flex-col gap-6"
      >
        <p className="max-w-3xl text-lg">{explanations[scheme]}</p>

        <ul className="grid grid-cols-[repeat(auto-fill,minmax(170px,1fr))] gap-3">
          {colors.map((c, i) => (
            <li
              key={`${i}-${c.hex}`}
              className="flex flex-col gap-2 rounded-lg border border-line bg-surface p-3"
            >
              <Swatch hex={c.hex} className="h-20 w-full rounded-md" />
              <span className="text-xs font-semibold tracking-wider text-muted uppercase">
                {ROLE_LABEL[c.role]}
                {c.locked && " · locked"}
              </span>
              <span className="leading-tight font-semibold">
                {headlineName(c.info, nameLevel).name}
              </span>
              <span className="font-mono text-sm text-muted">{c.hex}</span>
              <div className="mt-1 flex flex-wrap gap-2">
                <button
                  type="button"
                  aria-pressed={c.locked}
                  onClick={() =>
                    update(colors.map((x, j) => (j === i ? { ...x, locked: !x.locked } : x)))
                  }
                  className="rounded border border-line px-2 py-1 text-sm hover:border-accent aria-pressed:bg-raised aria-pressed:font-semibold"
                >
                  {c.locked ? "🔒 Locked" : "Lock"}
                </button>
                <button
                  type="button"
                  aria-expanded={editing === i}
                  onClick={() => setEditing(editing === i ? null : i)}
                  className="rounded border border-line px-2 py-1 text-sm hover:border-accent"
                >
                  Replace
                </button>
              </div>
              {editing === i && (
                <ColorInputBar
                  label={`Replace ${ROLE_LABEL[c.role].toLowerCase()} colour`}
                  placeholder="Hex, RGB or name"
                  showPicker={false}
                  onSelect={(hex) => void replace(i, hex)}
                />
              )}
            </li>
          ))}
        </ul>

        <div>
          <button
            type="button"
            onClick={() => void regenerate()}
            className="rounded-md border border-line bg-surface px-3 py-2 font-semibold hover:border-accent"
          >
            Regenerate unlocked colours
          </button>
        </div>

        <section aria-labelledby="save-heading" className="flex flex-col gap-3">
          <h2 id="save-heading" className="text-xl font-extrabold">
            Save and export
          </h2>
          <div className="flex flex-wrap items-center gap-2">
            <label htmlFor="palette-name" className="font-semibold">
              Name
            </label>
            <input
              id="palette-name"
              type="text"
              value={name}
              placeholder={paletteName}
              onChange={(e) => setName(e.target.value)}
              className="w-72 max-w-full rounded-md border border-line bg-surface px-3 py-2 placeholder:text-muted"
            />
            <button
              type="button"
              onClick={() => void save(false)}
              className="rounded-md bg-accent px-3 py-2 font-semibold text-on-accent"
            >
              {saved ? "Save changes" : "Save palette"}
            </button>
            {saved && (
              <button
                type="button"
                onClick={() => void save(true)}
                className="rounded-md border border-line bg-surface px-3 py-2 font-semibold hover:border-accent"
              >
                Save as new
              </button>
            )}
            <span role="status" className="font-semibold">
              {saveStatus}
            </span>
          </div>
          <ExportPanel
            palette={{
              name: paletteName,
              scheme,
              baseHex: base,
              colors: colors.map(({ hex, role }) => ({ hex, role })),
            }}
          />
        </section>

        <section aria-labelledby="check-heading" className="flex flex-col gap-3">
          <h2 id="check-heading" className="text-xl font-extrabold">
            Can everyone tell these apart?
          </h2>
          <p className="max-w-3xl text-muted">
            Every pair as seen with typical vision and each colour vision type at full strength.
            Contrast is shown as a WCAG level for text pairs.
          </p>
          <CheckGrid
            colors={colors.map((c) => ({
              hex: c.hex,
              role: c.role,
              name: headlineName(c.info, nameLevel).name,
            }))}
            checks={checks}
            userVision={vision}
          />
        </section>
      </section>
    </div>
  );
}
