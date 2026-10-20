//! Pairwise distinguishability under each vision type, and "confused with" hints.

use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

use crate::convert::Lab;
use crate::delta_e::ciede2000;
use crate::naming::{ColorName, Names};
use crate::palette::Role;
use crate::simulate::{VisionProfile, VisionType, simulate_lab};
use crate::{Rgb, contrast};

/// Typical vision: ΔE2000 at or above this is clearly different…
pub const DISTINCT_DE: f64 = 15.0;
/// …and at or above this (below [`DISTINCT_DE`]) tellable apart with care.
pub const CAUTION_DE: f64 = 8.0;
/// Simulated colour vision deficiencies use stricter thresholds: simulation is an approximation
/// and people vary, so leave a margin (decided at review checkpoint 3).
pub const CVD_DISTINCT_DE: f64 = 20.0;
pub const CVD_CAUTION_DE: f64 = 10.0;
/// Severity used for every deficiency in palette checks: the worst case, so a palette that
/// passes works for all colourblind viewers, not only the user.
pub const CHECK_SEVERITY: f64 = 1.0;

/// "Confused with": simulated colours closer than this look alike…
pub const CONFUSED_SIMULATED_DE: f64 = 5.0;
/// …while being at least this far apart for typical vision.
pub const CONFUSED_ACTUAL_DE: f64 = DISTINCT_DE;
const CONFUSED_MAX: usize = 6;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PairVerdict {
    Distinct,
    Caution,
    HardToTell,
}

/// Verdict for a ΔE2000 seen with `vision`.
pub fn verdict(delta_e: f64, vision: VisionType) -> PairVerdict {
    let (distinct, caution) = match vision {
        VisionType::Normal => (DISTINCT_DE, CAUTION_DE),
        _ => (CVD_DISTINCT_DE, CVD_CAUTION_DE),
    };
    if delta_e >= distinct {
        PairVerdict::Distinct
    } else if delta_e >= caution {
        PairVerdict::Caution
    } else {
        PairVerdict::HardToTell
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PairCheck {
    /// Indexes into the checked colours.
    pub a: usize,
    pub b: usize,
    /// ΔE2000 between the two colours as seen with each vision type (all of
    /// [`VisionType::ALL`], deficiencies at [`CHECK_SEVERITY`]).
    pub results: Vec<(VisionType, f32, PairVerdict)>,
    /// WCAG 2.x contrast ratio.
    pub contrast_ratio: f32,
    /// Background and surface that form a subtle tint, so they're meant to look alike; never
    /// counted as a warning. See [`is_subtle_tint`].
    pub subtle: bool,
}

/// Background and surface are meant to be a subtle tint of each other.
pub fn is_subtle_pair(a: Role, b: Role) -> bool {
    matches!(
        (a, b),
        (Role::Background, Role::Surface) | (Role::Surface, Role::Background)
    )
}

/// Below this typical-vision ΔE2000 a background and surface are the same colour, which hides
/// the surface rather than tinting it.
pub const SUBTLE_MIN_DE: f64 = 1.0;

/// A background/surface pair counts as a subtle tint only while it is one: visibly different
/// for typical vision, but less than [`DISTINCT_DE`] apart. Identical or replaced-with-anything
/// pairs are checked like every other pair (decided at review checkpoint 6).
pub fn is_subtle_tint(a: Role, b: Role, normal_delta_e: f64) -> bool {
    is_subtle_pair(a, b) && (SUBTLE_MIN_DE..DISTINCT_DE).contains(&normal_delta_e)
}

/// Every pair of `colors`, under normal vision and each deficiency. `roles`, when given (one per
/// colour), marks a background/surface tint as [`PairCheck::subtle`].
pub fn check_palette(colors: &[Rgb], roles: Option<&[Role]>) -> Vec<PairCheck> {
    let seen: Vec<Vec<Lab>> = colors
        .iter()
        .map(|&c| {
            VisionType::ALL
                .iter()
                .map(|&v| simulate_lab(c, v, CHECK_SEVERITY))
                .collect()
        })
        .collect();

    let mut out = Vec::new();
    for a in 0..colors.len() {
        for b in a + 1..colors.len() {
            let results: Vec<(VisionType, f32, PairVerdict)> = VisionType::ALL
                .iter()
                .enumerate()
                .map(|(i, &v)| {
                    let de = ciede2000(seen[a][i], seen[b][i]);
                    (v, de as f32, verdict(de, v))
                })
                .collect();
            out.push(PairCheck {
                a,
                b,
                subtle: roles.is_some_and(|r| is_subtle_tint(r[a], r[b], results[0].1 as f64)),
                results,
                contrast_ratio: contrast::contrast_ratio(colors[a], colors[b]) as f32,
            });
        }
    }
    out
}

/// Named (CSS) colours that look almost the same as `rgb` with this vision profile but are
/// clearly different for typical vision and belong to another basic colour family.
/// Closest look-alikes first. Empty for normal vision.
pub fn confused_with(rgb: Rgb, profile: VisionProfile) -> Vec<ColorName> {
    if profile.vision == VisionType::Normal || profile.severity <= 0.0 {
        return Vec::new();
    }
    let severity = profile.severity as f64;
    let lab = rgb.to_lab();
    let own_family = basic_family(lab);
    let seen = simulate_lab(rgb, profile.vision, severity);

    let mut hits: Vec<(f64, &ColorName)> = candidates()
        .iter()
        .filter(|c| c.family != own_family && ciede2000(lab, c.lab) >= CONFUSED_ACTUAL_DE)
        .filter_map(|c| {
            let de = ciede2000(seen, simulate_lab(c.rgb, profile.vision, severity));
            (de < CONFUSED_SIMULATED_DE).then_some((de, &c.name))
        })
        .collect();
    hits.sort_by(|a, b| a.0.total_cmp(&b.0));
    hits.into_iter()
        .take(CONFUSED_MAX)
        .map(|(de, name)| ColorName {
            delta_e: de as f32,
            ..name.clone()
        })
        .collect()
}

struct Candidate {
    name: ColorName,
    rgb: Rgb,
    lab: Lab,
    family: String,
}

/// The descriptive (CSS) names with their basic family, computed once.
fn candidates() -> &'static [Candidate] {
    static CANDIDATES: OnceLock<Vec<Candidate>> = OnceLock::new();
    CANDIDATES.get_or_init(|| {
        Names::bundled()
            .descriptive
            .colors()
            .map(|(name, lab)| Candidate {
                rgb: Rgb::from_hex(&name.hex).expect("list hex"),
                family: basic_family(lab),
                name,
                lab,
            })
            .collect()
    })
}

fn basic_family(lab: Lab) -> String {
    Names::bundled().basic.nearest(lab).name
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex(h: &str) -> Rgb {
        Rgb::from_hex(h).unwrap()
    }

    fn result(check: &PairCheck, v: VisionType) -> (f32, PairVerdict) {
        let r = check.results.iter().find(|r| r.0 == v).unwrap();
        (r.1, r.2)
    }

    #[test]
    fn red_and_green_are_flagged_for_protan() {
        let checks = check_palette(&[hex("#FF0000"), hex("#008000")], None);
        assert_eq!(checks.len(), 1);
        assert_eq!(
            result(&checks[0], VisionType::Normal).1,
            PairVerdict::Distinct
        );
        assert_ne!(
            result(&checks[0], VisionType::Protan).1,
            PairVerdict::Distinct
        );
    }

    /// SPEC Phase 3 acceptance: #FF0000 vs #008000 under deutan is Caution or HardToTell
    /// (ΔE 15.7: distinct for typical-vision thresholds, caution with the CVD margin).
    #[test]
    fn red_and_green_are_flagged_for_deutan() {
        let checks = check_palette(&[hex("#FF0000"), hex("#008000")], None);
        assert_ne!(
            result(&checks[0], VisionType::Deutan).1,
            PairVerdict::Distinct
        );
    }

    #[test]
    fn black_and_white_are_distinct_for_everyone() {
        let checks = check_palette(&[hex("#000000"), hex("#FFFFFF")], None);
        for (_, _, v) in &checks[0].results {
            assert_eq!(*v, PairVerdict::Distinct);
        }
        assert!((checks[0].contrast_ratio - 21.0).abs() < 1e-3);
    }

    #[test]
    fn checks_every_pair_once() {
        let colors = [
            hex("#FF0000"),
            hex("#00FF00"),
            hex("#0000FF"),
            hex("#FFFFFF"),
        ];
        let pairs: Vec<(usize, usize)> = check_palette(&colors, None)
            .iter()
            .map(|c| (c.a, c.b))
            .collect();
        assert_eq!(pairs, [(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)]);
    }

    #[test]
    fn thresholds() {
        use VisionType::*;
        assert_eq!(verdict(15.0, Normal), PairVerdict::Distinct);
        assert_eq!(verdict(8.0, Normal), PairVerdict::Caution);
        assert_eq!(verdict(7.9, Normal), PairVerdict::HardToTell);
        for v in [Protan, Deutan, Tritan, Achromat] {
            assert_eq!(verdict(20.0, v), PairVerdict::Distinct);
            assert_eq!(verdict(15.0, v), PairVerdict::Caution);
            assert_eq!(verdict(10.0, v), PairVerdict::Caution);
            assert_eq!(verdict(9.9, v), PairVerdict::HardToTell);
        }
    }

    #[test]
    fn background_and_surface_are_marked_subtle() {
        use Role::*;
        let colors = [hex("#F3F9FF"), hex("#DEEDFB"), hex("#4682B4")];
        let checks = check_palette(&colors, Some(&[Background, Surface, Primary]));
        let subtle: Vec<bool> = checks.iter().map(|c| c.subtle).collect();
        assert_eq!(subtle, [true, false, false]);
        assert!(check_palette(&colors, None).iter().all(|c| !c.subtle));
    }

    #[test]
    fn identical_or_clashing_surfaces_are_checked_normally() {
        use Role::*;
        let roles = [Background, Surface];
        // Same colour: the surface disappears, so it counts.
        let same = check_palette(&[hex("#F3F9FF"), hex("#F3F9FF")], Some(&roles));
        assert!(!same[0].subtle);
        // Replaced with something that no longer reads as a tint: counts.
        let clash = check_palette(&[hex("#F3F9FF"), hex("#C0392B")], Some(&roles));
        assert!(!clash[0].subtle);
    }

    #[test]
    fn red_is_confused_with_greens_or_browns_for_deutan() {
        let profile = VisionProfile {
            vision: VisionType::Deutan,
            severity: 1.0,
        };
        let names = confused_with(hex("#B22222"), profile);
        assert!(!names.is_empty());
        for n in &names {
            assert!(n.delta_e < CONFUSED_SIMULATED_DE as f32);
            let family = basic_family(hex(&n.hex).to_lab());
            assert_ne!(family, "red", "{n:?}");
        }
    }

    #[test]
    fn nothing_is_confused_with_normal_vision() {
        let profile = VisionProfile {
            vision: VisionType::Normal,
            severity: 1.0,
        };
        assert!(confused_with(hex("#FF0000"), profile).is_empty());
    }
}
