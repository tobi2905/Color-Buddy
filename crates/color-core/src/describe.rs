//! Plain-language description of a colour from its OKLCH values,
//! e.g. "light, slightly desaturated, cool blue".

use crate::Rgb;
use crate::convert::Oklch;

/// Below this OKLCH chroma a colour reads as neutral (grey/black/white).
const NEUTRAL_CHROMA: f64 = 0.02;

/// Lightness word for OKLCH L.
pub fn lightness_word(l: f64) -> &'static str {
    match l {
        l if l < 0.30 => "very dark",
        l if l < 0.45 => "dark",
        l if l < 0.65 => "medium",
        l if l < 0.82 => "light",
        _ => "very light",
    }
}

/// OKLCH chroma treated as fully saturated when judging saturation in absolute terms.
const VIVID_CHROMA: f64 = 0.15;

/// Saturation score 0–1: chroma relative to the most saturated sRGB colour at the same L and h,
/// capped by absolute chroma so pale colours (where the gamut is narrow) aren't called vivid.
pub fn saturation(lch: Oklch) -> f64 {
    let relative = lch.c / max_chroma(lch.l, lch.h).max(1e-9);
    relative.min(lch.c / VIVID_CHROMA).min(1.0)
}

/// Saturation word for a [`saturation`] score.
pub fn chroma_word(saturation: f64) -> &'static str {
    match saturation {
        r if r < 0.15 => "greyish",
        r if r < 0.40 => "muted",
        r if r < 0.75 => "slightly desaturated",
        _ => "vivid",
    }
}

/// "warm", "cool", or `None` for hues in between (yellow-greens, purples).
pub fn temperature_word(hue: f64) -> Option<&'static str> {
    match hue {
        h if !(100.0..345.0).contains(&h) => Some("warm"), // pinks, reds, oranges, yellows
        h if (130.0..300.0).contains(&h) => Some("cool"),  // greens, cyans, blues, violets
        _ => None,
    }
}

/// Highest chroma that is still inside sRGB at this lightness and hue.
pub fn max_chroma(l: f64, h: f64) -> f64 {
    let (mut lo, mut hi) = (0.0, 0.5);
    for _ in 0..30 {
        let mid = (lo + hi) / 2.0;
        if (Oklch { l, c: mid, h }).in_gamut() {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    lo
}

/// Describe `rgb`, using `basic` (its basic colour name) as the noun.
pub fn describe(rgb: Rgb, basic: &str) -> String {
    let lch = rgb.to_oklch();
    let lightness = lightness_word(lch.l);

    if lch.c < NEUTRAL_CHROMA {
        return match (basic, rgb) {
            (
                _,
                Rgb {
                    r: 255,
                    g: 255,
                    b: 255,
                },
            ) => "pure white".into(),
            (_, Rgb { r: 0, g: 0, b: 0 }) => "pure black".into(),
            ("white" | "black", _) => format!("neutral {basic}"),
            _ => format!("{lightness} neutral {basic}"),
        };
    }

    let chroma = chroma_word(saturation(lch));
    match temperature_word(lch.h) {
        Some(temp) => format!("{lightness}, {chroma}, {temp} {basic}"),
        None => format!("{lightness}, {chroma} {basic}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn neutrals() {
        assert_eq!(describe(Rgb::new(255, 255, 255), "white"), "pure white");
        assert_eq!(describe(Rgb::new(0, 0, 0), "black"), "pure black");
        assert_eq!(
            describe(Rgb::new(128, 128, 128), "grey"),
            "medium neutral grey"
        );
        assert_eq!(describe(Rgb::new(250, 250, 250), "white"), "neutral white");
    }

    #[test]
    fn primaries_are_vivid() {
        assert_eq!(
            describe(Rgb::new(255, 0, 0), "red"),
            "medium, vivid, warm red"
        );
        assert!(describe(Rgb::new(0, 0, 255), "blue").contains("vivid, cool blue"));
    }

    #[test]
    fn pale_colours_are_not_vivid() {
        assert_eq!(
            describe(Rgb::new(0xFF, 0xC0, 0xCB), "pink"),
            "very light, slightly desaturated, warm pink"
        );
    }

    #[test]
    fn sky_blue() {
        assert_eq!(
            describe(Rgb::new(0x87, 0xCE, 0xEB), "blue"),
            "light, slightly desaturated, cool blue"
        );
    }

    #[test]
    fn greyish_colours() {
        // Slate grey: a blue-tinted grey.
        assert_eq!(
            describe(Rgb::new(0x70, 0x80, 0x90), "grey"),
            "medium, muted, cool grey"
        );
        // Dark khaki-ish grey with very little colour.
        assert!(describe(Rgb::new(0x80, 0x80, 0x78), "grey").contains("neutral"));
    }

    #[test]
    fn max_chroma_is_on_the_gamut_boundary() {
        let red = Rgb::new(255, 0, 0).to_oklch();
        assert!((max_chroma(red.l, red.h) - red.c).abs() < 1e-3);
    }
}
