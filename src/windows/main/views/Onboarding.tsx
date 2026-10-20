import { useEffect, useState } from "react";
import { HotkeyRecorder } from "../../../components/HotkeyRecorder";
import { ProfilePicker } from "../../../components/ProfilePicker";
import { formatAccelerator, isMac } from "../../../lib/accelerator";
import { getScreenPermission, requestScreenPermission } from "../../../lib/app";
import { openScreenRecordingSettings } from "../../../lib/sampler";
import { useSettings } from "../../../store/settings";

type Step = "welcome" | "vision" | "shortcut" | "permission";

/** First launch: explain the app, set the vision profile and shortcut, grant screen access. */
export function Onboarding({ mac = isMac }: { mac?: boolean }) {
  const s = useSettings();
  const steps: Step[] = mac
    ? ["welcome", "vision", "shortcut", "permission"]
    : ["welcome", "vision", "shortcut"];
  const [index, setIndex] = useState(0);
  const step = steps[index];
  const last = index === steps.length - 1;

  return (
    <main className="mx-auto flex min-h-screen max-w-2xl flex-col justify-center gap-8 px-6 py-10">
      <p className="text-sm text-muted">
        Step {index + 1} of {steps.length}
      </p>

      {step === "welcome" && (
        <section className="flex flex-col gap-4" aria-labelledby="ob-title">
          <h1 id="ob-title" className="text-4xl font-extrabold text-balance">
            Welcome to Color-Buddy
          </h1>
          <p className="text-lg">
            Pick, name and combine colours with confidence, without relying on your own colour
            perception.
          </p>
          <ul className="flex flex-col gap-2 text-lg">
            <li>
              <span aria-hidden="true">◎ </span>Point at anything on screen to hear what colour it
              is, in plain words.
            </li>
            <li>
              <span aria-hidden="true">▦ </span>Build palettes that explain why they work.
            </li>
            <li>
              <span aria-hidden="true">✓ </span>Check that every colour stays tellable apart, for
              you and for others.
            </li>
          </ul>
        </section>
      )}

      {step === "vision" && (
        <section className="flex flex-col gap-4" aria-labelledby="ob-title">
          <h1 id="ob-title" className="text-3xl font-extrabold">
            Your colour vision
          </h1>
          <p className="text-muted">Used to warn you about colours that may look alike to you.</p>
          <ProfilePicker
            profile={s.profile}
            unsure={s.profileUnsure}
            onChange={(profile, profileUnsure) => s.update({ profile, profileUnsure })}
          />
        </section>
      )}

      {step === "shortcut" && (
        <section className="flex flex-col gap-4" aria-labelledby="ob-title">
          <h1 id="ob-title" className="text-3xl font-extrabold">
            Picker shortcut
          </h1>
          <p className="text-lg">
            Press <strong>{formatAccelerator(s.hotkey)}</strong> in any app to open the picker,
            press it again (or click) to copy the colour, and once more to close.
          </p>
          <HotkeyRecorder accelerator={s.hotkey} onChange={(hotkey) => s.update({ hotkey })} />
        </section>
      )}

      {step === "permission" && <PermissionStep />}

      <nav className="flex gap-3" aria-label="Welcome steps">
        {index > 0 && (
          <button
            type="button"
            onClick={() => setIndex(index - 1)}
            className="rounded-md border border-line bg-surface px-4 py-2 font-semibold hover:border-accent"
          >
            Back
          </button>
        )}
        <button
          type="button"
          onClick={() => (last ? s.update({ onboarded: true }) : setIndex(index + 1))}
          className="rounded-md bg-accent px-4 py-2 font-semibold text-on-accent"
        >
          {last ? "Start using Color-Buddy" : "Next"}
        </button>
      </nav>
    </main>
  );
}

function PermissionStep() {
  const [granted, setGranted] = useState<boolean | null>(null);
  useEffect(() => {
    getScreenPermission().then(setGranted, () => setGranted(false));
  }, []);

  return (
    <section className="flex flex-col gap-4" aria-labelledby="ob-title">
      <h1 id="ob-title" className="text-3xl font-extrabold">
        Screen access
      </h1>
      <p className="text-lg">
        macOS asks before an app can read colours from the screen. Color-Buddy only looks at the few
        pixels under the cursor while the picker is open, and nothing leaves your computer.
      </p>
      {granted ? (
        <p className="font-semibold text-ok">
          <span aria-hidden="true">✓ </span>Screen access is on.
        </p>
      ) : (
        <>
          <ol className="list-decimal pl-6 text-lg">
            <li>Choose “Allow screen access”.</li>
            <li>Turn on Color-Buddy under Screen &amp; System Audio Recording.</li>
            <li>Quit and reopen Color-Buddy so the change takes effect.</li>
          </ol>
          <button
            type="button"
            onClick={() => {
              void requestScreenPermission();
              void openScreenRecordingSettings();
            }}
            className="self-start rounded-md border border-line bg-surface px-4 py-2 font-semibold hover:border-accent"
          >
            Allow screen access
          </button>
          <p className="text-sm text-muted">You can also do this later from Settings.</p>
        </>
      )}
    </section>
  );
}
