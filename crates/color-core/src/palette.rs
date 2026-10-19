//! Palette schemes generated in OKLCH, with roles and a plain-language explanation.

use serde::{Deserialize, Serialize};

use crate::convert::Oklch;
use crate::describe::max_chroma;
use crate::{ColorInfo, Rgb, color_info};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Scheme {
    Complementary,
    Analogous,
    Triadic,
    SplitComplementary,
    Monochrome,
}

impl Scheme {
    pub const ALL: [Scheme; 5] = [
        Scheme::Complementary,
        Scheme::Analogous,
        Scheme::Triadic,
        Scheme::SplitComplementary,
        Scheme::Monochrome,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Role {
    Background,
    Surface,
    Primary,
    Accent,
    Text,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PaletteColor {
    pub hex: String,
    pub role: Role,
    pub info: ColorInfo,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Palette {
    pub scheme: Scheme,
    pub base_hex: String,
    /// Background, surface, primary, accent(s), text — in that order.
    pub colors: Vec<PaletteColor>,
    /// Why it works, in one or two plain sentences.
    pub explanation: String,
}

// Lightness and chroma of the neutral roles (OKLCH). Text on background is ~15:1 for any hue.
const BACKGROUND: (f64, f64) = (0.98, 0.012);
const SURFACE: (f64, f64) = (0.94, 0.025);
const TEXT: (f64, f64) = (0.25, 0.035);
/// Accents of a near-neutral base get this chroma, so the scheme still has hues.
const NEUTRAL_BASE_CHROMA: f64 = 0.03;
const NEUTRAL_ACCENT_CHROMA: f64 = 0.10;
/// Accent lightness range, so accents of very light or dark bases stay usable.
const ACCENT_L: (f64, f64) = (0.30, 0.90);
/// OKLCH lightness step between the primary and each accent. Accents differ in lightness, not
/// only hue, so they stay apart for colourblind viewers (decided at review checkpoint 3).
const LIGHTNESS_STEP: f64 = 0.2;

/// The closest in-gamut colour: same L and h, chroma reduced until it fits sRGB.
pub fn gamut_map(lch: Oklch) -> Rgb {
    let l = lch.l.clamp(0.0, 1.0);
    let c = lch.c.min(max_chroma(l, lch.h)).max(0.0);
    Oklch { l, c, h: lch.h }.to_rgb()
}

/// All five schemes for `base`.
pub fn generate_palettes(base: Rgb) -> Vec<Palette> {
    Scheme::ALL.iter().map(|&s| generate(base, s)).collect()
}

pub fn generate(base: Rgb, scheme: Scheme) -> Palette {
    let b = base.to_oklch();
    let neutral = b.c < NEUTRAL_BASE_CHROMA;
    let accent_c = if neutral { NEUTRAL_ACCENT_CHROMA } else { b.c };
    let at = |dh: f64, l: f64| {
        gamut_map(Oklch {
            l,
            c: accent_c,
            h: (b.h + dh).rem_euclid(360.0),
        })
    };
    let tint = |(l, c): (f64, f64)| {
        gamut_map(Oklch {
            l,
            c: c.min(b.c),
            h: b.h,
        })
    };

    let hue_steps: &[f64] = match scheme {
        Scheme::Complementary => &[180.0],
        Scheme::Analogous => &[-30.0, 30.0],
        Scheme::Triadic => &[120.0, 240.0],
        Scheme::SplitComplementary => &[150.0, 210.0],
        Scheme::Monochrome => &[0.0, 0.0],
    };
    let lightness = accent_lightness(b.l, hue_steps.len());
    let accents: Vec<Rgb> = hue_steps
        .iter()
        .zip(lightness)
        .map(|(&dh, l)| {
            if scheme == Scheme::Monochrome {
                gamut_map(Oklch { l, c: b.c, h: b.h })
            } else {
                at(dh, l)
            }
        })
        .collect();

    let mut colors = vec![
        (tint(BACKGROUND), Role::Background),
        (tint(SURFACE), Role::Surface),
        (base, Role::Primary),
    ];
    colors.extend(accents.iter().map(|&a| (a, Role::Accent)));
    colors.push((tint(TEXT), Role::Text));

    let colors: Vec<PaletteColor> = colors
        .into_iter()
        .map(|(rgb, role)| PaletteColor {
            hex: rgb.to_hex(),
            role,
            info: color_info(rgb),
        })
        .collect();

    Palette {
        scheme,
        base_hex: base.to_hex(),
        explanation: explain(scheme, &colors, neutral),
        colors,
    }
}

/// Lightness of each accent, a [`LIGHTNESS_STEP`] or more away from the primary's `base_l`.
/// One accent goes the way with more room; two go either side when both fit, otherwise two
/// steps the same way. Clamped to [`ACCENT_L`].
fn accent_lightness(base_l: f64, count: usize) -> Vec<f64> {
    let (lo, hi) = ACCENT_L;
    let clamp = |l: f64| l.clamp(lo, hi);
    let room_up = hi - base_l;
    let room_down = base_l - lo;
    let dir = if room_up >= room_down { 1.0 } else { -1.0 };
    match count {
        0 => Vec::new(),
        1 => vec![clamp(base_l + dir * LIGHTNESS_STEP)],
        _ if room_up >= LIGHTNESS_STEP && room_down >= LIGHTNESS_STEP => {
            vec![base_l - LIGHTNESS_STEP, base_l + LIGHTNESS_STEP]
        }
        _ => vec![
            clamp(base_l + dir * LIGHTNESS_STEP),
            clamp(base_l + dir * 2.0 * LIGHTNESS_STEP),
        ],
    }
}

fn explain(scheme: Scheme, colors: &[PaletteColor], neutral_base: bool) -> String {
    let name = |c: &PaletteColor| c.info.descriptive.name.clone();
    let primary = colors
        .iter()
        .find(|c| c.role == Role::Primary)
        .map(name)
        .unwrap();
    let accents: Vec<String> = colors
        .iter()
        .filter(|c| c.role == Role::Accent)
        .map(name)
        .collect();
    let both = |a: &[String]| match a {
        [x] => x.clone(),
        [x, y] => format!("{x} and {y}"),
        _ => a.join(", "),
    };

    let mut text = match scheme {
        Scheme::Complementary => format!(
            "{primary} with its opposite, {}: the strongest hue contrast, good for making one action stand out.",
            both(&accents)
        ),
        Scheme::Analogous => format!(
            "{primary} with its neighbours {}: calm and cohesive, because the hues sit side by side.",
            both(&accents)
        ),
        Scheme::Triadic => format!(
            "{primary}, {}, evenly spaced around the colour wheel: lively but balanced.",
            both(&accents)
        ),
        Scheme::SplitComplementary => format!(
            "{primary} with {}, either side of its opposite: strong contrast that is easier to balance than a straight complement.",
            both(&accents)
        ),
        Scheme::Monochrome => format!(
            "{primary} in lighter and darker steps ({}): one hue throughout, so contrast comes from lightness alone, which works for every colour vision type.",
            both(&accents)
        ),
    };
    if scheme != Scheme::Monochrome {
        text.push_str(
            " The accents are also lighter or darker, so they stay apart without relying on hue.",
        );
    }
    if neutral_base && scheme != Scheme::Monochrome {
        text.push_str(" The base is nearly grey, so the accents add the colour.");
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contrast::contrast_ratio;

    fn grid() -> impl Iterator<Item = Rgb> {
        (0..=255)
            .step_by(51)
            .flat_map(|r| (0..=255).step_by(51).map(move |g| (r, g)))
            .flat_map(|(r, g)| {
                (0..=255)
                    .step_by(51)
                    .map(move |b| Rgb::new(r as u8, g as u8, b as u8))
            })
    }

    #[test]
    fn every_scheme_has_the_expected_roles() {
        let p = generate_palettes(Rgb::new(0x46, 0x82, 0xB4));
        let roles = |s: Scheme| -> Vec<Role> {
            p.iter()
                .find(|x| x.scheme == s)
                .unwrap()
                .colors
                .iter()
                .map(|c| c.role)
                .collect()
        };
        use Role::*;
        assert_eq!(
            roles(Scheme::Complementary),
            [Background, Surface, Primary, Accent, Text]
        );
        assert_eq!(
            roles(Scheme::Triadic),
            [Background, Surface, Primary, Accent, Accent, Text]
        );
        assert_eq!(
            roles(Scheme::Monochrome),
            [Background, Surface, Primary, Accent, Accent, Text]
        );
    }

    #[test]
    fn primary_is_the_base_colour() {
        for p in generate_palettes(Rgb::new(0x87, 0xCE, 0xEB)) {
            assert_eq!(p.base_hex, "#87CEEB");
            let primary = p.colors.iter().find(|c| c.role == Role::Primary).unwrap();
            assert_eq!(primary.hex, "#87CEEB");
        }
    }

    #[test]
    fn complementary_accent_is_opposite_hue() {
        let base = Rgb::new(0x46, 0x82, 0xB4);
        let p = generate(base, Scheme::Complementary);
        let accent = p.colors.iter().find(|c| c.role == Role::Accent).unwrap();
        let h = Rgb::from_hex(&accent.hex).unwrap().to_oklch().h;
        let want = (base.to_oklch().h + 180.0) % 360.0;
        let diff = (h - want).abs().min(360.0 - (h - want).abs());
        assert!(diff < 3.0, "accent hue {h}, want {want}");
    }

    #[test]
    fn all_colours_are_valid_srgb_and_text_is_readable() {
        for base in grid() {
            for p in generate_palettes(base) {
                for c in &p.colors {
                    let rgb = Rgb::from_hex(&c.hex).expect("valid hex");
                    // Gamut mapping keeps every colour within 1 step of its ideal value.
                    assert!(rgb.to_linear().in_gamut());
                }
                let get = |r: Role| {
                    Rgb::from_hex(&p.colors.iter().find(|c| c.role == r).unwrap().hex).unwrap()
                };
                let ratio = contrast_ratio(get(Role::Text), get(Role::Background));
                assert!(
                    ratio >= 7.0,
                    "{:?} {}: text/background {ratio:.1}",
                    p.scheme,
                    p.base_hex
                );
            }
        }
    }

    /// Accents are offset in lightness so that, across many bases, primary/accent pairs are
    /// rarely hard to tell apart for colourblind viewers.
    #[test]
    fn primary_and_accents_stay_apart_for_colourblind_viewers() {
        use crate::check::{PairVerdict, check_palette};
        use crate::simulate::VisionType;
        let (mut hard, mut total) = (0, 0);
        for base in grid() {
            for p in generate_palettes(base) {
                let rgbs: Vec<Rgb> = p
                    .colors
                    .iter()
                    .map(|c| Rgb::from_hex(&c.hex).unwrap())
                    .collect();
                let roles: Vec<Role> = p.colors.iter().map(|c| c.role).collect();
                for c in check_palette(&rgbs, Some(&roles)) {
                    if roles[c.a] != Role::Primary || roles[c.b] != Role::Accent {
                        continue;
                    }
                    for (v, _, verdict) in &c.results {
                        if matches!(
                            v,
                            VisionType::Protan | VisionType::Deutan | VisionType::Tritan
                        ) {
                            total += 1;
                            hard += usize::from(*verdict == PairVerdict::HardToTell);
                        }
                    }
                }
            }
        }
        let rate = hard as f64 / total as f64;
        assert!(
            rate < 0.05,
            "{hard}/{total} primary/accent checks hard to tell ({:.1}%)",
            rate * 100.0
        );
    }

    #[test]
    fn gamut_map_reduces_chroma_only() {
        let out = gamut_map(Oklch {
            l: 0.9,
            c: 0.3,
            h: 260.0,
        })
        .to_oklch();
        assert!((out.l - 0.9).abs() < 0.01);
        assert!((out.h - 260.0).abs() < 2.0);
        assert!(out.c < 0.3);
    }

    #[test]
    fn explanations_name_the_colours() {
        let p = generate(Rgb::new(0x87, 0xCE, 0xEB), Scheme::Complementary);
        assert!(
            p.explanation.starts_with("sky blue with its opposite"),
            "{}",
            p.explanation
        );
    }
}
