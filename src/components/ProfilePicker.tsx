import { useId } from "react";
import { SEVERITY_OPTIONS, UNSURE_PROFILE, VISION_OPTIONS } from "../store/settings";
import type { VisionProfile } from "../types/color";

// Radio, title and hint are direct children of the label (grid), so its text is easy to find.
const card =
  "grid cursor-pointer grid-cols-[auto_1fr] items-baseline gap-x-3 rounded-md border border-line bg-surface p-3 has-[:checked]:border-accent has-[:checked]:bg-raised";

/** Vision type + severity, as native radio groups (keyboard: arrows within each group). */
export function ProfilePicker({
  profile,
  unsure,
  onChange,
}: {
  profile: VisionProfile;
  unsure: boolean;
  onChange: (profile: VisionProfile, unsure: boolean) => void;
}) {
  const name = useId();
  return (
    <div className="flex flex-col gap-5">
      <fieldset className="flex flex-col gap-2">
        <legend className="mb-2 font-semibold">Type of colour vision</legend>
        <div className="grid gap-2 sm:grid-cols-2">
          {VISION_OPTIONS.map((o) => (
            <label key={o.value} className={card}>
              <input
                type="radio"
                name={`${name}-vision`}
                checked={!unsure && profile.vision === o.value}
                onChange={() =>
                  onChange(
                    {
                      vision: o.value,
                      severity: o.value === "normal" ? 0 : profile.severity || 0.6,
                    },
                    false,
                  )
                }
                className="accent-[var(--app-accent)]"
              />
              <span className="font-semibold">{o.label}</span>
              <span className="col-start-2 text-sm text-muted">{o.hint}</span>
            </label>
          ))}
          <label className={card}>
            <input
              type="radio"
              name={`${name}-vision`}
              checked={unsure}
              onChange={() => onChange(UNSURE_PROFILE, true)}
              className="accent-[var(--app-accent)]"
            />
            <span className="font-semibold">Not sure</span>
            <span className="col-start-2 text-sm text-muted">
              Uses moderate deutan, the most common type. You can change it any time.
            </span>
          </label>
        </div>
      </fieldset>

      <fieldset className="flex flex-col gap-2" disabled={unsure || profile.vision === "normal"}>
        <legend className="mb-2 font-semibold">How strong is it?</legend>
        <div className="flex flex-wrap gap-2">
          {SEVERITY_OPTIONS.map((o) => (
            <label key={o.value} className={`${card} items-center py-2`}>
              <input
                type="radio"
                name={`${name}-severity`}
                checked={Math.abs(profile.severity - o.value) < 0.05}
                onChange={() => onChange({ ...profile, severity: o.value }, false)}
                className="accent-[var(--app-accent)]"
              />
              <span className="font-semibold">{o.label}</span>
            </label>
          ))}
        </div>
      </fieldset>

      <p className="text-sm text-muted">
        This only tunes the app's warnings. It isn't a medical test; an eye-care professional can
        tell you your exact type.
      </p>
    </div>
  );
}
