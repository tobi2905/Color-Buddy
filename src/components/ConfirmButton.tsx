import { useState, type ReactNode } from "react";

/** A destructive action with an in-page confirmation step (no system dialogs). */
export function ConfirmButton({
  children,
  confirmLabel,
  question,
  onConfirm,
  className = "",
}: {
  children: ReactNode;
  confirmLabel: string;
  question: string;
  onConfirm: () => void | Promise<void>;
  className?: string;
}) {
  const [asking, setAsking] = useState(false);
  const base = "rounded-md border px-3 py-1.5 font-semibold";
  if (!asking) {
    return (
      <button
        type="button"
        onClick={() => setAsking(true)}
        className={`${base} border-line bg-surface hover:border-accent ${className}`}
      >
        {children}
      </button>
    );
  }
  return (
    <span role="group" aria-label={question} className="inline-flex flex-wrap items-center gap-2">
      <span className="font-semibold">{question}</span>
      <button
        type="button"
        onClick={async () => {
          await onConfirm();
          setAsking(false);
        }}
        className={`${base} border-bad bg-bad-bg text-bad`}
      >
        <span aria-hidden="true">✕ </span>
        {confirmLabel}
      </button>
      <button
        type="button"
        onClick={() => setAsking(false)}
        className={`${base} border-line bg-surface hover:border-accent`}
      >
        Cancel
      </button>
    </span>
  );
}
