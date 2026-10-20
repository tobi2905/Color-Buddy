import type { ReactNode } from "react";
import { ConfirmButton } from "../../../components/ConfirmButton";
import { HotkeyRecorder } from "../../../components/HotkeyRecorder";
import { ProfilePicker } from "../../../components/ProfilePicker";
import {
  useSettings,
  type NameLevel,
  type SampleSize,
  type Theme,
  type ValueFormat,
} from "../../../store/settings";
import SamplerDebug from "../SamplerDebug";

const card =
  "grid cursor-pointer grid-cols-[auto_1fr] items-baseline gap-x-3 rounded-md border border-line bg-surface p-3 has-[:checked]:border-accent has-[:checked]:bg-raised";

function Choice<T extends string | number>({
  legend,
  name,
  value,
  options,
  onChange,
}: {
  legend: string;
  name: string;
  value: T;
  options: { value: T; label: string; hint?: string }[];
  onChange: (value: T) => void;
}) {
  return (
    <fieldset className="flex flex-col gap-2">
      <legend className="mb-2 font-semibold">{legend}</legend>
      <div className="flex flex-wrap gap-2">
        {options.map((o) => (
          <label key={String(o.value)} className={card}>
            <input
              type="radio"
              name={name}
              checked={value === o.value}
              onChange={() => onChange(o.value)}
              className="accent-[var(--app-accent)]"
            />
            <span className="font-semibold">{o.label}</span>
            {o.hint && <span className="col-start-2 text-sm text-muted">{o.hint}</span>}
          </label>
        ))}
      </div>
    </fieldset>
  );
}

function Section({ title, children }: { title: string; children: ReactNode }) {
  return (
    <section className="flex flex-col gap-3 border-t border-line pt-6">
      <h2 className="text-xl font-extrabold">{title}</h2>
      {children}
    </section>
  );
}

export function SettingsView() {
  const s = useSettings();
  return (
    <div className="flex max-w-3xl flex-col gap-6">
      <h1 className="text-3xl font-extrabold">Settings</h1>

      <Section title="Your colour vision">
        <ProfilePicker
          profile={s.profile}
          unsure={s.profileUnsure}
          onChange={(profile, profileUnsure) => s.update({ profile, profileUnsure })}
        />
      </Section>

      <Section title="Picker shortcut">
        <p className="text-muted">
          Opens the picker from any app. Press it again to copy, and once more to close.
        </p>
        <HotkeyRecorder accelerator={s.hotkey} onChange={(hotkey) => s.update({ hotkey })} />
      </Section>

      <Section title="Picker">
        <Choice<SampleSize>
          legend="Sample size"
          name="sample-size"
          value={s.sampleSize}
          onChange={(sampleSize) => s.update({ sampleSize })}
          options={[
            { value: 1, label: "1 × 1", hint: "Exact pixel" },
            { value: 3, label: "3 × 3", hint: "Small area" },
            { value: 5, label: "5 × 5", hint: "Larger loupe" },
          ]}
        />
        <Choice<NameLevel>
          legend="Name shown first"
          name="name-level"
          value={s.nameLevel}
          onChange={(nameLevel) => s.update({ nameLevel })}
          options={[
            { value: "basic", label: "Basic", hint: "e.g. blue" },
            { value: "descriptive", label: "Descriptive", hint: "e.g. sky blue" },
            { value: "precise", label: "Precise", hint: "e.g. light blue" },
          ]}
        />
      </Section>

      <Section title="Colour values">
        <Choice<ValueFormat>
          legend="Show colour values as"
          name="value-format"
          value={s.valueFormat}
          onChange={(valueFormat) => s.update({ valueFormat })}
          options={[
            { value: "hex", label: "Hex", hint: "#87CEEB" },
            { value: "rgb", label: "RGB", hint: "rgb(135, 206, 235)" },
            { value: "hsl", label: "HSL", hint: "hsl(197, 71%, 73%)" },
            { value: "oklch", label: "OKLCH", hint: "oklch(81.4% 0.088 225.3)" },
          ]}
        />
        <p className="text-sm text-muted">
          Used on palette cards, saved palettes, history and suggestions, and when you click a value
          to copy it. Colour details always shows every format.
        </p>
      </Section>

      <Section title="Appearance">
        <Choice<Theme>
          legend="Theme"
          name="theme"
          value={s.theme}
          onChange={(theme) => s.update({ theme })}
          options={[
            { value: "system", label: "System", hint: "Follow the computer" },
            { value: "light", label: "Light" },
            { value: "dark", label: "Dark" },
          ]}
        />
      </Section>

      <Section title="Reset">
        <p className="text-muted">
          Puts every setting above back to its default, including your colour vision profile. Saved
          palettes and history stay.
        </p>
        <div>
          <ConfirmButton
            question="Reset all settings to their defaults?"
            confirmLabel="Reset settings"
            onConfirm={() => s.reset()}
          >
            Reset settings to default
          </ConfirmButton>
        </div>
      </Section>

      <Section title="Diagnostics">
        <details className="rounded-md border border-line bg-surface">
          <summary className="cursor-pointer px-4 py-3 font-semibold">
            Screen sampling self-test and measurements
          </summary>
          <SamplerDebug />
        </details>
        <button
          type="button"
          onClick={() => s.update({ onboarded: false })}
          className="self-start rounded-md border border-line bg-surface px-3 py-2 font-semibold hover:border-accent"
        >
          Show the welcome steps again
        </button>
      </Section>
    </div>
  );
}
