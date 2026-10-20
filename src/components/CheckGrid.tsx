import { ROLE_LABEL, VISION_LABEL } from "../lib/names";
import type { PairCheck, Role, VisionType } from "../types/color";
import { ContrastBadge } from "./ContrastBadge";
import { Swatch } from "./Swatch";
import { VerdictBadge } from "./VerdictBadge";

export interface GridColor {
  hex: string;
  role: Role;
  name: string;
}

const VISIONS: VisionType[] = ["normal", "protan", "deutan", "tritan", "achromat"];

/** Text pairs, where WCAG text contrast matters most. */
const isTextPair = (a: Role, b: Role) => (a === "text") !== (b === "text");

/** Summary per vision type + a pairs × vision types grid with contrast. */
export function CheckGrid({
  colors,
  checks,
  userVision,
}: {
  colors: GridColor[];
  checks: PairCheck[];
  userVision: VisionType;
}) {
  const counted = checks.filter((c) => !c.subtle);
  const summary = VISIONS.map((vision, i) => {
    const hard = counted.filter((c) => c.results[i][2] === "hardToTell").length;
    const caution = counted.filter((c) => c.results[i][2] === "caution").length;
    return { vision, hard, caution };
  });

  return (
    <div className="flex flex-col gap-4">
      <ul className="flex flex-wrap gap-2" aria-label="Summary by colour vision type">
        {summary.map(({ vision, hard, caution }) => {
          const verdict = hard ? "hardToTell" : caution ? "caution" : "distinct";
          return (
            <li
              key={vision}
              className="flex items-center gap-2 rounded-md border border-line bg-surface px-3 py-1.5"
            >
              <span className="font-semibold">
                {VISION_LABEL[vision]}
                {vision === userVision && (
                  <>
                    {" "}
                    <span className="font-normal text-muted">(you)</span>
                  </>
                )}
              </span>
              <VerdictBadge verdict={verdict} />
              <span className="text-sm text-muted">
                {hard ? `${hard} hard to tell` : caution ? `${caution} to check` : "all pairs"}
              </span>
            </li>
          );
        })}
      </ul>

      <div className="overflow-x-auto rounded-md border border-line">
        <table className="w-full border-collapse text-left text-sm">
          <caption className="sr-only">
            Distinguishability of every colour pair, as seen with each colour vision type
          </caption>
          <thead className="bg-raised">
            <tr>
              <th scope="col" className="px-3 py-2">
                Pair
              </th>
              {VISIONS.map((v) => (
                <th key={v} scope="col" className="px-3 py-2 whitespace-nowrap">
                  {VISION_LABEL[v]}
                  {v === userVision && (
                    <>
                      {" "}
                      <span className="font-normal text-muted">(you)</span>
                    </>
                  )}
                </th>
              ))}
              <th scope="col" className="px-3 py-2">
                Contrast
              </th>
            </tr>
          </thead>
          <tbody>
            {checks.map((c) => {
              const a = colors[c.a];
              const b = colors[c.b];
              if (!a || !b) return null;
              return (
                <tr
                  key={`${c.a}-${c.b}`}
                  className={`border-t border-line ${c.subtle ? "text-muted" : ""}`}
                >
                  <th scope="row" className="px-3 py-2 text-left font-normal">
                    <span className="flex items-center gap-2">
                      <Swatch hex={a.hex} size={16} />
                      <Swatch hex={b.hex} size={16} />
                      <span>
                        {ROLE_LABEL[a.role]} × {ROLE_LABEL[b.role]}
                        {c.subtle && <span className="block text-xs">Subtle by design</span>}
                      </span>
                    </span>
                  </th>
                  {c.results.map(([vision, de, verdict]) => (
                    <td key={vision} className="px-3 py-2">
                      <VerdictBadge verdict={verdict} deltaE={de} />
                    </td>
                  ))}
                  <td className="px-3 py-2">
                    {isTextPair(a.role, b.role) ? (
                      <ContrastBadge ratio={c.contrastRatio} />
                    ) : (
                      <span className="font-mono text-muted">{c.contrastRatio.toFixed(2)}:1</span>
                    )}
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>
    </div>
  );
}
