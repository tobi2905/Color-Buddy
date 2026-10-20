import { useEffect, useRef, useState } from "react";
import { acceleratorFromEvent, formatAccelerator } from "../lib/accelerator";
import { setHotkey } from "../lib/app";

/** Shows the picker shortcut and records a new one (registered immediately). */
export function HotkeyRecorder({
  accelerator,
  onChange,
}: {
  accelerator: string;
  onChange: (accelerator: string) => void;
}) {
  const [recording, setRecording] = useState(false);
  const [message, setMessage] = useState<string | null>(null);
  const listenRef = useRef<HTMLButtonElement>(null);
  useEffect(() => {
    if (recording) listenRef.current?.focus();
  }, [recording]);

  async function apply(next: string) {
    try {
      await setHotkey(next);
      onChange(next);
      setMessage(null);
    } catch (err) {
      setMessage(String(err));
    }
    setRecording(false);
  }

  return (
    <div className="flex flex-col gap-2">
      <div className="flex flex-wrap items-center gap-3">
        <kbd
          className="rounded-md border border-line bg-raised px-3 py-1.5 font-mono text-lg"
          aria-label={`Current shortcut: ${formatAccelerator(accelerator)}`}
        >
          {formatAccelerator(accelerator)}
        </kbd>
        {recording ? (
          <button
            ref={listenRef}
            type="button"
            onKeyDown={(e) => {
              e.preventDefault();
              if (e.key === "Escape") {
                setRecording(false);
                setMessage(null);
                return;
              }
              const result = acceleratorFromEvent(e);
              if (result.kind === "accelerator") void apply(result.accelerator);
              else if (result.kind === "invalid") setMessage(result.message);
            }}
            onBlur={() => setRecording(false)}
            className="rounded-md border-2 border-dashed border-accent bg-surface px-3 py-1.5 font-semibold"
          >
            Press the new shortcut… (Esc to cancel)
          </button>
        ) : (
          <button
            type="button"
            onClick={() => {
              setRecording(true);
              setMessage(null);
            }}
            className="rounded-md border border-line bg-surface px-3 py-1.5 font-semibold hover:border-accent"
          >
            Change shortcut
          </button>
        )}
      </div>
      {message && (
        <p role="alert" className="text-sm font-semibold text-bad">
          ⚠ {message}
        </p>
      )}
    </div>
  );
}
