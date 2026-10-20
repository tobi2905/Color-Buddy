import { useEffect, useState } from "react";
import { Swatch } from "../../../components/Swatch";
import { getColorInfo } from "../../../lib/color";
import { headlineName } from "../../../lib/names";
import { listHistory, onPickerFrozen, type HistoryEntry } from "../../../lib/picker";
import { timeAgo } from "../../../lib/time";
import { useApp } from "../../../store/app";
import { useSettings } from "../../../store/settings";
import type { ColorInfo } from "../../../types/color";

const SHOWN = 50;
/** Colours picked from the screen, newest first. */
export function HistoryView() {
  const selectColor = useApp((s) => s.selectColor);
  const nameLevel = useSettings((s) => s.nameLevel);
  const [entries, setEntries] = useState<(HistoryEntry & { info: ColorInfo })[] | null>(null);

  useEffect(() => {
    let current = true;
    const refresh = () =>
      listHistory(SHOWN)
        .then((list) =>
          Promise.all(list.map((e) => getColorInfo(e.hex))).then((infos) =>
            list.map((e, i) => ({ ...e, info: infos[i] })),
          ),
        )
        .then((next) => current && setEntries(next));
    void refresh();
    const off = onPickerFrozen(() => void refresh());
    return () => {
      current = false;
      void off.then((f) => f());
    };
  }, []);

  return (
    <div className="flex max-w-3xl flex-col gap-4">
      <header className="flex flex-col gap-1">
        <h1 className="text-3xl font-extrabold">History</h1>
        <p className="text-muted">
          Colours you copied with the picker. Kept until you quit the app for now; saving them comes
          with the next update.
        </p>
      </header>
      {entries === null ? (
        <p className="text-muted">Loading…</p>
      ) : entries.length === 0 ? (
        <p className="text-muted">
          Nothing picked yet. Use “Pick from screen” or the shortcut, then click to copy a colour.
        </p>
      ) : (
        <ol className="flex flex-col divide-y divide-line rounded-md border border-line bg-surface">
          {entries.map((e) => (
            <li key={e.id}>
              <button
                type="button"
                onClick={() => selectColor(e.hex)}
                className="flex w-full items-center gap-4 px-4 py-2 text-left hover:bg-raised"
              >
                <Swatch hex={e.hex} size={28} />
                <span className="flex-1 font-semibold">
                  {headlineName(e.info, nameLevel).name}
                </span>{" "}
                <span className="font-mono text-muted">{e.hex}</span>
                <span className="w-32 text-right text-sm text-muted">{timeAgo(e.pickedAt)}</span>
              </button>
            </li>
          ))}
        </ol>
      )}
    </div>
  );
}
