import { useRef, type ReactNode } from "react";

interface Tab<T extends string> {
  id: T;
  label: ReactNode;
}

/** WAI-ARIA tabs: arrow keys move between tabs, Home/End jump to the ends. */
export function Tabs<T extends string>({
  tabs,
  selected,
  onSelect,
  label,
  idPrefix,
}: {
  tabs: Tab<T>[];
  selected: T;
  onSelect: (id: T) => void;
  label: string;
  idPrefix: string;
}) {
  const refs = useRef<(HTMLButtonElement | null)[]>([]);
  const move = (index: number) => {
    const i = (index + tabs.length) % tabs.length;
    onSelect(tabs[i].id);
    refs.current[i]?.focus();
  };
  return (
    <div role="tablist" aria-label={label} className="flex flex-wrap gap-1 border-b border-line">
      {tabs.map((tab, i) => {
        const active = tab.id === selected;
        return (
          <button
            key={tab.id}
            ref={(el) => {
              refs.current[i] = el;
            }}
            type="button"
            role="tab"
            id={`${idPrefix}-tab-${tab.id}`}
            aria-selected={active}
            aria-controls={`${idPrefix}-panel`}
            tabIndex={active ? 0 : -1}
            onClick={() => onSelect(tab.id)}
            onKeyDown={(e) => {
              if (e.key === "ArrowRight") move(i + 1);
              else if (e.key === "ArrowLeft") move(i - 1);
              else if (e.key === "Home") move(0);
              else if (e.key === "End") move(tabs.length - 1);
              else return;
              e.preventDefault();
            }}
            className={`-mb-px border-b-[3px] px-3 py-2 text-sm font-semibold ${
              active ? "border-accent text-ink" : "border-transparent text-muted hover:text-ink"
            }`}
          >
            {tab.label}
          </button>
        );
      })}
    </div>
  );
}
