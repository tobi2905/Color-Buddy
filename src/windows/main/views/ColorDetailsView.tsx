import { useEffect, useState } from "react";
import { CopyValue } from "../../../components/CopyValue";
import { Swatch } from "../../../components/Swatch";
import { getColorDetails, getColorInfo } from "../../../lib/color";
import { headlineName, matchLabel } from "../../../lib/names";
import { useApp } from "../../../store/app";
import { profileLabel, useSettings } from "../../../store/settings";
import type { ColorDetails, ColorInfo } from "../../../types/color";

/** Everything about the current colour, in plain language first and numbers second. */
export function ColorDetailsView() {
  const hex = useApp((s) => s.currentHex);
  const selectColor = useApp((s) => s.selectColor);
  const profile = useSettings((s) => s.profile);
  const nameLevel = useSettings((s) => s.nameLevel);
  const [details, setDetails] = useState<ColorDetails | null>(null);
  const [pairings, setPairings] = useState<ColorInfo[]>([]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let current = true;
    getColorDetails(hex, profile).then(
      async (d) => {
        if (!current) return;
        setDetails(d);
        setError(null);
        const infos = await Promise.all(d.pairings.map((p) => getColorInfo(p)));
        if (current) setPairings(infos);
      },
      (err) => current && setError(String(err)),
    );
    return () => {
      current = false;
    };
  }, [hex, profile]);

  if (error) {
    return (
      <p role="alert" className="font-semibold text-bad">
        ⚠ {error}
      </p>
    );
  }
  if (!details) return <p className="text-muted">Loading {hex}…</p>;

  const { info } = details;
  const headline = headlineName(info, nameLevel);
  const confidence = matchLabel(headline.deltaE);

  return (
    <article className="flex max-w-4xl flex-col gap-8" aria-labelledby="color-headline">
      <header className="flex flex-wrap items-start gap-6">
        <Swatch
          hex={info.hex}
          className="h-36 w-36 rounded-lg"
          label={`Swatch of ${headline.name}, ${info.hex}`}
        />
        <div className="flex min-w-0 flex-col gap-1">
          <h1 id="color-headline" className="text-4xl font-extrabold text-balance">
            {headline.name}
            {confidence && (
              <span className="ml-3 align-middle text-base font-normal text-muted">
                {confidence}
              </span>
            )}
          </h1>
          <p className="text-lg">{info.description}</p>
          <p className="flex flex-col items-start gap-0.5 text-muted">
            <CopyValue label="hex" value={info.values.hex} />
            <CopyValue label="RGB" value={info.values.rgb} />
            <CopyValue label="HSL" value={info.values.hsl} />
            <CopyValue label="OKLCH" value={info.values.oklch} />
          </p>
          <button
            type="button"
            onClick={() => selectColor(info.hex, "palettes")}
            className="mt-3 self-start rounded-md bg-accent px-3 py-2 font-semibold text-on-accent"
          >
            Build a palette from this colour
          </button>
        </div>
      </header>

      <section aria-labelledby="names-heading" className="flex flex-col gap-2">
        <h2 id="names-heading" className="text-xl font-extrabold">
          Names
        </h2>
        <dl className="grid grid-cols-[max-content_1fr] gap-x-6 gap-y-1">
          <dt className="text-muted">Basic</dt>
          <dd className="font-semibold">
            {info.basic.name}
            {info.basicAlternative && (
              <span className="font-normal">, bordering on {info.basicAlternative.name}</span>
            )}
          </dd>
          <dt className="text-muted">Descriptive</dt>
          <dd>
            <span className="font-semibold">{info.descriptive.name}</span>{" "}
            <NameDistance deltaE={info.descriptive.deltaE} />
          </dd>
          <dt className="text-muted">Precise</dt>
          <dd>
            <span className="font-semibold">{info.precise.name}</span>{" "}
            <NameDistance deltaE={info.precise.deltaE} />
          </dd>
        </dl>
      </section>

      <section aria-labelledby="world-heading" className="flex flex-col gap-2">
        <h2 id="world-heading" className="text-xl font-extrabold">
          In the real world
        </h2>
        <p className="text-sm text-muted">From our notes on {details.referenceSource.label}.</p>
        <dl className="grid grid-cols-[max-content_1fr] gap-x-6 gap-y-1">
          <dt className="text-muted">Looks like</dt>
          <dd>{details.references.join(", ")}</dd>
          <dt className="text-muted">Feels and used for</dt>
          <dd>{details.associations.join(", ")}</dd>
        </dl>
      </section>

      <section aria-labelledby="pairs-heading" className="flex flex-col gap-2">
        <h2 id="pairs-heading" className="text-xl font-extrabold">
          Pairs well with
        </h2>
        <ColorChips colors={pairings} nameLevel={nameLevel} onSelect={selectColor} />
      </section>

      <section aria-labelledby="confused-heading" className="flex flex-col gap-2">
        <h2 id="confused-heading" className="text-xl font-extrabold">
          May look alike to you
        </h2>
        {details.confusedWith.length ? (
          <>
            <p className="text-muted">
              With {profileLabel(profile)}, these named colours can look almost the same as this
              one, although they clearly differ for most people.
            </p>
            <ul className="flex flex-wrap gap-2">
              {details.confusedWith.map((n) => (
                <li key={`${n.name}-${n.hex}`}>
                  <button
                    type="button"
                    onClick={() => selectColor(n.hex)}
                    className="flex items-center gap-2 rounded-md border border-line bg-surface px-3 py-1.5 hover:border-accent"
                  >
                    <Swatch hex={n.hex} />
                    <span className="font-semibold">{n.name}</span>{" "}
                    <span className="font-mono text-sm text-muted">{n.hex}</span>
                  </button>
                </li>
              ))}
            </ul>
          </>
        ) : profile.vision === "normal" ? (
          <p className="text-muted">
            Your profile is typical colour vision, so there are no look-alike warnings. Palettes
            still check every colour vision type for your audience.
          </p>
        ) : (
          <p className="text-muted">
            <span aria-hidden="true">✓ </span>No common look-alikes for {profileLabel(profile)}.
          </p>
        )}
      </section>
    </article>
  );
}

function NameDistance({ deltaE }: { deltaE: number }) {
  const label = matchLabel(deltaE);
  return (
    <span className="text-sm text-muted">
      {label ?? "exact"} · <span className="font-mono">ΔE {deltaE.toFixed(1)}</span>
    </span>
  );
}

export function ColorChips({
  colors,
  nameLevel,
  onSelect,
}: {
  colors: ColorInfo[];
  nameLevel: "basic" | "descriptive" | "precise";
  onSelect: (hex: string) => void;
}) {
  return (
    <ul className="flex flex-wrap gap-2">
      {colors.map((p) => (
        <li key={p.hex}>
          <button
            type="button"
            onClick={() => onSelect(p.hex)}
            className="flex items-center gap-2 rounded-md border border-line bg-surface px-3 py-1.5 hover:border-accent"
          >
            <Swatch hex={p.hex} />
            <span className="font-semibold">{headlineName(p, nameLevel).name}</span>{" "}
            <span className="font-mono text-sm text-muted">{p.hex}</span>
          </button>
        </li>
      ))}
    </ul>
  );
}
