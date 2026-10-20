import { useEffect, useRef, useState } from "react";
import { copyToClipboard } from "../lib/storage";

/** A value that copies itself when clicked, with a short spoken/visible confirmation. */
export function CopyValue({
  value,
  label,
  compact = false,
}: {
  value: string;
  label: string;
  /** Short confirmation ("Copied") for narrow spots such as palette cards. */
  compact?: boolean;
}) {
  const [status, setStatus] = useState<string | null>(null);
  const timer = useRef<ReturnType<typeof setTimeout>>(undefined);
  useEffect(() => () => clearTimeout(timer.current), []);

  async function copy() {
    try {
      await copyToClipboard(value);
      setStatus(compact ? "Copied" : `Copied ${value}`);
    } catch (err) {
      setStatus(`Couldn't copy: ${String(err)}`);
    }
    clearTimeout(timer.current);
    timer.current = setTimeout(() => setStatus(null), 1800);
  }

  return (
    <span className="inline-flex flex-wrap items-baseline gap-x-2">
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
