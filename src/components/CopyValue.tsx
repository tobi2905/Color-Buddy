import { useEffect, useRef, useState } from "react";
import { copyToClipboard } from "../lib/storage";

/** A value that copies itself when clicked, with a short spoken/visible confirmation. */
export function CopyValue({ value, label }: { value: string; label: string }) {
  const [status, setStatus] = useState<string | null>(null);
  const timer = useRef<ReturnType<typeof setTimeout>>(undefined);
  useEffect(() => () => clearTimeout(timer.current), []);

  async function copy() {
    try {
      await copyToClipboard(value);
      setStatus(`Copied ${value}`);
    } catch (err) {
      setStatus(`Couldn't copy: ${String(err)}`);
    }
    clearTimeout(timer.current);
    timer.current = setTimeout(() => setStatus(null), 1800);
  }

  return (
    <span className="inline-flex items-baseline gap-2">
      <button
        type="button"
        onClick={() => void copy()}
        aria-label={`Copy ${label} ${value}`}
        className="rounded px-1 font-mono underline decoration-dotted underline-offset-4 hover:bg-raised"
      >
        {value}
      </button>
      <span role="status" className="text-sm font-semibold">
        {status && (
          <>
            <span aria-hidden="true">✓ </span>
            {status}
          </>
        )}
      </span>
    </span>
  );
}
