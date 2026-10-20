import { useEffect } from "react";
import { ColorInputBar } from "../../components/ColorInputBar";
import { setHotkey } from "../../lib/app";
import { onPickerFrozen } from "../../lib/picker";
import { setSampleSize } from "../../lib/sampler";
import { useApp, type View } from "../../store/app";
import { useSettings, type Theme } from "../../store/settings";
import { ColorDetailsView } from "./views/ColorDetailsView";
import { HistoryView } from "./views/HistoryView";
import { LibraryView } from "./views/LibraryView";
import { Onboarding } from "./views/Onboarding";
import { PaletteBuilderView } from "./views/PaletteBuilderView";
import { SettingsView } from "./views/SettingsView";

const NAV: { id: View; label: string; icon: string }[] = [
  { id: "color", label: "Colour", icon: "◎" },
  { id: "palettes", label: "Palettes", icon: "▦" },
  { id: "saved", label: "Saved palettes", icon: "★" },
  { id: "history", label: "History", icon: "↺" },
  { id: "settings", label: "Settings", icon: "⚙" },
];

function useTheme(theme: Theme) {
  useEffect(() => {
    const media = window.matchMedia?.("(prefers-color-scheme: dark)");
    const apply = () => {
      const dark = theme === "dark" || (theme === "system" && !!media?.matches);
      document.documentElement.classList.toggle("dark", dark);
    };
    apply();
    media?.addEventListener?.("change", apply);
    return () => media?.removeEventListener?.("change", apply);
  }, [theme]);
}

/** Push saved settings to the backend on start and whenever they change. */
function useBackendSettings() {
  const hotkey = useSettings((s) => s.hotkey);
  const sampleSize = useSettings((s) => s.sampleSize);
  useEffect(() => {
    setHotkey(hotkey).catch((err) => console.warn("hotkey:", err));
  }, [hotkey]);
  useEffect(() => {
    setSampleSize(sampleSize).catch((err) => console.warn("sample size:", err));
  }, [sampleSize]);
}

function MainWindow() {
  const theme = useSettings((s) => s.theme);
  const onboarded = useSettings((s) => s.onboarded);
  const { view, setView, selectColor } = useApp();
  useTheme(theme);
  useBackendSettings();

  // A colour copied with the picker opens in Colour details.
  useEffect(() => {
    const off = onPickerFrozen((e) => selectColor(e.info.hex));
    return () => void off.then((f) => f());
  }, [selectColor]);

  if (!onboarded) return <Onboarding />;

  return (
    <div className="flex h-screen bg-app text-ink">
      <nav aria-label="Main" className="flex w-52 shrink-0 flex-col gap-1 border-r border-line p-3">
        <p className="px-3 pt-1 pb-4 text-lg font-extrabold">Color-Buddy</p>
        {NAV.map((n) => (
          <button
            key={n.id}
            type="button"
            aria-current={view === n.id ? "page" : undefined}
            onClick={() => setView(n.id)}
            className="flex items-center gap-3 rounded-md px-3 py-2 text-left font-semibold hover:bg-raised aria-[current=page]:bg-raised aria-[current=page]:shadow-[inset_3px_0_0_var(--app-accent)]"
          >
            <span aria-hidden="true" className="w-5 text-center">
              {n.icon}
            </span>
            {n.label}
          </button>
        ))}
      </nav>
      <div className="flex min-w-0 flex-1 flex-col">
        <header className="border-b border-line bg-surface px-6 py-3">
          <ColorInputBar onSelect={(hex) => selectColor(hex)} />
        </header>
        <main className="min-h-0 flex-1 overflow-auto px-6 py-6">
          {view === "color" && <ColorDetailsView />}
          {view === "palettes" && <PaletteBuilderView />}
          {view === "saved" && <LibraryView />}
          {view === "history" && <HistoryView />}
          {view === "settings" && <SettingsView />}
        </main>
      </div>
    </div>
  );
}

export default MainWindow;
