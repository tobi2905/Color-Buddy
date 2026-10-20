import { useEffect, useRef, useState } from "react";
import {
  getSamplerStatus,
  onColorHovered,
  onSamplerStatus,
  openScreenRecordingSettings,
  selfTest,
  setSampleSize,
  startPicker,
  stopPicker,
} from "../../lib/sampler";
import { listHistory, onPickerFrozen, type HistoryEntry } from "../../lib/picker";
import type { ColorInfo } from "../../types/color";
import type { PatchResult, SamplerStatus } from "../../types/sampler";
import { TEST_PATCHES } from "./testPatches";

const SIZES = [1, 3, 5];
const isMac = typeof navigator !== "undefined" && navigator.userAgent.includes("Mac");
const HOTKEY = isMac ? "⌘ Shift C" : "Ctrl + Shift + C";

/** Diagnostics for screen sampling (Settings → Diagnostics): self-test and measurements. */
function SamplerDebug() {
  const [info, setInfo] = useState<ColorInfo | null>(null);
  const [history, setHistory] = useState<HistoryEntry[]>([]);
  const hex = info?.hex ?? null;
  const [status, setStatus] = useState<SamplerStatus | null>(null);
  const [results, setResults] = useState<PatchResult[] | null>(null);
  const [testError, setTestError] = useState<string | null>(null);
  const patchRefs = useRef<(HTMLDivElement | null)[]>([]);

  useEffect(() => {
    const refreshHistory = () => listHistory(10).then(setHistory, () => {});
    const unlisten = [
      onColorHovered((e) => setInfo(e.info)),
      onSamplerStatus(setStatus),
      onPickerFrozen(() => void refreshHistory()),
    ];
    void refreshHistory();
    getSamplerStatus().then(setStatus, () => {});
    return () => unlisten.forEach((p) => p.then((off) => off()));
  }, []);

  // `VITE_SELFTEST=1 npm run tauri dev` runs the self-test once the window has rendered
  // (after `VITE_SELFTEST_DELAY_MS`, default 1500); results are logged by the backend.
  useEffect(() => {
    if (import.meta.env.VITE_SELFTEST !== "1") return;
    const delay = Number(import.meta.env.VITE_SELFTEST_DELAY_MS ?? 1500);
    const timer = setTimeout(() => void runSelfTest(), delay);
    return () => clearTimeout(timer);
  }, []);

  const running = status?.running ?? false;
  const size = status?.size ?? 5;

  async function changeSize(next: number) {
    await setSampleSize(next);
    setStatus(await getSamplerStatus());
  }

  async function runSelfTest() {
    setTestError(null);
    setResults(null);
    const patches = TEST_PATCHES.map((patchHex, i) => {
      const rect = patchRefs.current[i]!.getBoundingClientRect();
      return { hex: patchHex, x: rect.left + rect.width / 2, y: rect.top + rect.height / 2 };
    });
    try {
      setResults(await selfTest(patches));
    } catch (err) {
      setTestError(String(err));
    }
  }

  return (
    <div className="flex flex-col gap-6 p-4">
      <p className="text-muted">
        Tests and measurements for screen sampling. Open the picker with{" "}
        <kbd className="rounded border border-line px-1">{HOTKEY}</kbd> or the button below.
      </p>

      <section aria-label="Sampler controls" className="flex flex-wrap items-center gap-3">
        <button
          type="button"
          className="rounded border border-neutral-500 px-3 py-1"
          onClick={() => (running ? stopPicker() : startPicker())}
        >
          {running ? "Stop sampling" : "Start sampling"}
        </button>
        <label className="flex items-center gap-2">
          Sample size
          <select
            className="rounded border border-neutral-500 bg-transparent px-2 py-1"
            value={size}
            onChange={(e) => changeSize(Number(e.target.value))}
          >
            {SIZES.map((n) => (
              <option key={n} value={n}>
                {n}×{n}
              </option>
            ))}
          </select>
        </label>
        <span>Status: {running ? "sampling" : "stopped"}</span>
      </section>

      {status?.error && (
        <section role="alert" className="rounded border-2 border-neutral-900 p-3 dark:border-white">
          <p className="font-semibold">⚠ {status.error}</p>
          {status.errorKind === "permission" && (
            <button
              type="button"
              className="mt-2 rounded border border-neutral-500 px-3 py-1"
              onClick={() => openScreenRecordingSettings()}
            >
              Open Screen Recording settings
            </button>
          )}
        </section>
      )}

      <section aria-label="Current color" className="flex items-center gap-4">
        <div
          className="h-20 w-20 rounded border border-neutral-500"
          style={{ backgroundColor: hex ?? "transparent" }}
          aria-hidden="true"
        />
        <div>
          <p className="text-sm text-muted">Under cursor</p>
          <p className="font-mono text-3xl" data-testid="hovered-hex">
            {hex ?? "—"}
          </p>
          {info && (
            <dl
              className="mt-1 grid grid-cols-[max-content_1fr] gap-x-3 text-sm"
              data-testid="hovered-names"
            >
              <dt>Basic</dt>
              <dd>
                {info.basic.name}
                {info.basicAlternative && ` (bordering on ${info.basicAlternative.name})`}
              </dd>
              <dt>Descriptive</dt>
              <dd>
                {info.descriptive.name} (ΔE {info.descriptive.deltaE.toFixed(1)})
              </dd>
              <dt>Precise</dt>
              <dd>
                {info.precise.name} (ΔE {info.precise.deltaE.toFixed(1)})
              </dd>
              <dt>Description</dt>
              <dd>{info.description}</dd>
            </dl>
          )}
        </div>
      </section>

      <section aria-label="Picked colours">
        <h2 className="text-lg font-semibold">Picked colours</h2>
        {history.length === 0 ? (
          <p className="text-muted">None yet. Open the picker and click to copy a colour.</p>
        ) : (
          <ol className="flex flex-wrap gap-2" data-testid="history">
            {history.map((h) => (
              <li key={h.id} className="flex items-center gap-2 rounded border px-2 py-1">
                <span
                  className="h-4 w-4 rounded-sm border border-neutral-500"
                  style={{ backgroundColor: h.hex }}
                  aria-hidden="true"
                />
                <span className="font-mono">{h.hex}</span>
              </li>
            ))}
          </ol>
        )}
      </section>

      <section aria-label="Measurements">
        <h2 className="text-lg font-semibold">Measurements</h2>
        <dl className="grid grid-cols-[max-content_1fr] gap-x-4 gap-y-1">
          <dt>Samples per second</dt>
          <dd>{status ? status.samplesPerSecond.toFixed(0) : "—"}</dd>
          <dt>Average capture time</dt>
          <dd>{status?.avgCaptureMs ? `${status.avgCaptureMs.toFixed(2)} ms` : "—"}</dd>
          <dt>Hotkey to first sample</dt>
          <dd>{status?.firstSampleMs != null ? `${status.firstSampleMs.toFixed(1)} ms` : "—"}</dd>
          <dt>Display</dt>
          <dd>
            {status?.display
              ? `#${status.display.id}, scale ${status.display.scale}×, profile: ${status.display.profile}` +
                (status.display.isSrgb
                  ? ""
                  : status.display.convertedToSrgb
                    ? " (not sRGB, converted to sRGB)"
                    : " (not sRGB, no correction)")
              : "—"}
          </dd>
        </dl>
      </section>

      <section aria-label="Test patches">
        <h2 className="text-lg font-semibold">Test patches</h2>
        <p className="text-muted">
          Hover each patch, or run the self-test to sample them automatically at 1×1, 3×3 and 5×5.
        </p>
        <div className="mt-3 flex flex-wrap gap-4">
          {TEST_PATCHES.map((patchHex, i) => (
            <figure key={patchHex} className="flex flex-col items-center gap-1">
              <div
                ref={(el) => {
                  patchRefs.current[i] = el;
                }}
                className="h-28 w-28"
                style={{ backgroundColor: patchHex }}
                aria-hidden="true"
              />
              <figcaption className="font-mono">{patchHex}</figcaption>
            </figure>
          ))}
        </div>
        <button
          type="button"
          className="mt-3 rounded border border-neutral-500 px-3 py-1"
          onClick={runSelfTest}
        >
          Run self-test
        </button>
        {testError && (
          <p role="alert" className="mt-2 font-semibold">
            ⚠ {testError}
          </p>
        )}
        {results && (
          <table className="mt-3 w-full text-left font-mono text-sm">
            <thead>
              <tr>
                <th>Expected</th>
                <th>Size</th>
                <th>Centre</th>
                <th>Max diff</th>
                <th>Result</th>
              </tr>
            </thead>
            <tbody>
              {results.map((r) => (
                <tr key={`${r.expected}-${r.size}`}>
                  <td>{r.expected}</td>
                  <td>
                    {r.size}×{r.size}
                  </td>
                  <td>{r.center}</td>
                  <td>{r.maxChannelDiff}</td>
                  <td>{r.allMatch ? "✓ exact" : "✕ mismatch"}</td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </section>
    </div>
  );
}

export default SamplerDebug;
