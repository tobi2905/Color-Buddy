import { useEffect, useLayoutEffect, useRef, useState } from "react";
import { freezePick, nudgeCursor, onPickerFrozen, onPickerState } from "../../lib/picker";
import { onColorHovered, stopPicker } from "../../lib/sampler";
import type { Rgb } from "../../types/color";
import type { HoveredEvent } from "../../types/sampler";
import { matchLabel, placePanel } from "./placement";

/** Width of the magnified loupe in CSS pixels, whatever the sample size. */
const LOUPE_PX = 96;
const ARROWS: Record<string, [number, number]> = {
  ArrowLeft: [-1, 0],
  ArrowRight: [1, 0],
  ArrowUp: [0, -1],
  ArrowDown: [0, 1],
};

const css = (c: Rgb) => `rgb(${c.r} ${c.g} ${c.b})`;

/** Overlay for one display: loupe + label next to the cursor, catches click and keys. */
function PickerOverlay({ display }: { display: number }) {
  const [hover, setHover] = useState<HoveredEvent | null>(null);
  const [frozen, setFrozen] = useState<HoveredEvent | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [size, setSize] = useState({ width: 280, height: 180 });
  const panelRef = useRef<HTMLDivElement>(null);
  // Event callbacks and key handlers are registered once; they read the frozen state here.
  const frozenRef = useRef(false);
  useEffect(() => {
    frozenRef.current = frozen !== null;
  }, [frozen]);

  useEffect(() => {
    const reset = () => {
      setFrozen(null);
      setError(null);
    };
    const unlisten = [
      onColorHovered((e) => {
        if (!frozenRef.current) setHover(e);
      }),
      onPickerFrozen((e) => {
        frozenRef.current = true;
        setHover(e);
        setFrozen(e);
        setError(null);
      }),
      onPickerState("picker-resumed", reset),
      onPickerState("picker-opened", () => {
        reset();
        setHover(null);
      }),
      onPickerState("picker-closed", () => {
        reset();
        setHover(null);
      }),
    ];
    return () => unlisten.forEach((p) => p.then((off) => off()));
  }, []);

  async function freezeOrClose() {
    if (frozenRef.current) {
      await stopPicker();
      return;
    }
    try {
      await freezePick();
    } catch (err) {
      setError(String(err));
    }
  }

  useEffect(() => {
    function onKey(e: KeyboardEvent) {
      if (e.key === "Escape") {
        e.preventDefault();
        void stopPicker();
      } else if (e.key === "Enter" || e.key === " ") {
        e.preventDefault();
        void freezeOrClose();
      } else if (e.key in ARROWS) {
        e.preventDefault();
        const [dx, dy] = ARROWS[e.key];
        const step = e.shiftKey ? 10 : 1;
        nudgeCursor(dx * step, dy * step).catch((err) => setError(String(err)));
      }
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  // Measure the panel so it can flip away from display edges.
  useLayoutEffect(() => {
    const el = panelRef.current;
    if (el && (el.offsetWidth !== size.width || el.offsetHeight !== size.height)) {
      setSize({ width: el.offsetWidth, height: el.offsetHeight });
    }
  }, [hover, frozen, error, size.width, size.height]);

  const shown = hover && hover.display === display ? hover : null;
  const isFrozen = frozen !== null && shown === frozen;

  return (
    <>
      {/* A full-display button: focusable and announced as a button. Enter/Space are handled
          (and cancelled) by the window key handler, so it never fires twice. */}
      <button
        type="button"
        className="block h-full w-full cursor-crosshair border-0 bg-transparent p-0 outline-none"
        aria-label="Copy the colour under the cursor. Escape closes, arrow keys move one pixel."
        data-testid="catcher"
        onMouseDown={(e) => {
          if (e.button === 0) void freezeOrClose();
        }}
      />
      {shown && (
        <div
          ref={panelRef}
          role="status"
          aria-live="polite"
          className="pointer-events-none absolute flex w-max max-w-[340px] gap-3 rounded-lg border border-white/30 bg-[#14171c] p-3 text-white shadow-[0_6px_24px_rgb(0_0_0/0.45)]"
          style={placePanel({ x: shown.localX, y: shown.localY }, size, {
            width: window.innerWidth,
            height: window.innerHeight,
          })}
        >
          <Loupe grid={shown.loupe} />
          <Label event={shown} frozen={isFrozen} error={error} />
        </div>
      )}
    </>
  );
}

function Loupe({ grid }: { grid: Rgb[][] }) {
  const n = grid.length;
  const cell = Math.floor(LOUPE_PX / Math.max(n, 1));
  const mid = Math.floor(n / 2);
  return (
    <div
      className="grid shrink-0 self-start border border-white/40"
      style={{ gridTemplateColumns: `repeat(${n}, ${cell}px)` }}
      aria-hidden="true"
      data-testid="loupe"
    >
      {grid.flatMap((row, y) =>
        row.map((c, x) => (
          <div
            key={`${x}-${y}`}
            style={{
              width: cell,
              height: cell,
              background: css(c),
              // Black inside white: visible on any colour.
              outline: x === mid && y === mid ? "2px solid black" : undefined,
              boxShadow: x === mid && y === mid ? "0 0 0 4px white" : undefined,
              position: "relative",
              zIndex: x === mid && y === mid ? 1 : 0,
            }}
          />
        )),
      )}
    </div>
  );
}

function Label({
  event,
  frozen,
  error,
}: {
  event: HoveredEvent;
  frozen: boolean;
  error: string | null;
}) {
  const { info } = event;
  const confidence = matchLabel(info.descriptive.deltaE);
  return (
    <div className="flex min-w-0 flex-col gap-0.5">
      <p className="text-xl leading-tight font-bold" data-testid="descriptive">
        {info.descriptive.name}
        {confidence && (
          <span className="ml-2 align-middle text-xs font-normal text-white/70">{confidence}</span>
        )}
      </p>
      <p className="text-sm text-white/85">
        {info.basic.name}
        {info.basicAlternative && `, bordering on ${info.basicAlternative.name}`}
        {" · "}
        <span className="font-mono">{info.hex}</span>
      </p>
      <p className="text-sm text-white/85">{info.description}</p>
      {frozen ? (
        <>
          <p className="mt-1 font-semibold" data-testid="toast">
            ✓ Copied {info.hex} ({info.descriptive.name})
          </p>
          <p className="text-xs text-white/70">Click or Esc to close · arrows keep picking</p>
        </>
      ) : (
        <p className="mt-1 text-xs text-white/70">
          Click or Enter to copy · Esc to close · arrows move 1 px
        </p>
      )}
      {error && (
        <p role="alert" className="mt-1 text-sm font-semibold">
          ⚠ {error}
        </p>
      )}
    </div>
  );
}

export default PickerOverlay;
