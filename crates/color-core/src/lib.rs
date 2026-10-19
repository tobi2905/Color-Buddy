//! Pure color logic for Color-Buddy.
//!
//! This crate has no Tauri dependency so it can be tested on its own. See `SPEC.md`.

pub mod check;
pub mod contrast;
pub mod convert;
pub mod delta_e;
pub mod describe;
#[rustfmt::skip]
mod machado_matrices;
pub mod naming;
pub mod palette;
pub mod references;
pub mod simulate;

use serde::{Deserialize, Serialize};

pub use naming::ColorName;

/// An 8-bit sRGB color.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Uppercase `#RRGGBB`.
    pub fn to_hex(self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }
}

/// When the second-closest basic term is within this CIEDE2000 distance of the closest, the
/// colour sits on a boundary and both names are shown. Calibrated on the Phase 2 sighted review:
/// every boundary colour where the reviewer chose "either" or the second term had a gap ≤ 1.5.
pub const BASIC_NEAR_TIE_DE: f32 = 2.0;

/// Everything we know about a single colour.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ColorInfo {
    pub hex: String,
    pub rgb: Rgb,
    /// OKLCH: L (0–1), C, h (degrees).
    pub oklch: [f32; 3],
    /// e.g. "blue" — the closest basic term.
    pub basic: ColorName,
    /// The second-closest basic term when it is nearly as close (see [`BASIC_NEAR_TIE_DE`]),
    /// e.g. "brown" for sienna, whose basic name is "orange".
    pub basic_alternative: Option<ColorName>,
    /// e.g. "sky blue" (CSS named colours)
    pub descriptive: ColorName,
    /// e.g. "light sky blue" (XKCD colour survey)
    pub precise: ColorName,
    /// e.g. "light, slightly desaturated, cool blue"; boundary colours end in
    /// ", bordering on <alternative>".
    pub description: String,
}

/// Names, OKLCH values and description for a colour.
pub fn color_info(rgb: Rgb) -> ColorInfo {
    let names = naming::Names::bundled();
    let lab = rgb.to_lab();
    let mut terms = names.basic.ranked_names(lab).into_iter();
    let basic = terms.next().expect("basic list is empty");
    let basic_alternative = terms
        .next()
        .filter(|alt| alt.delta_e - basic.delta_e <= BASIC_NEAR_TIE_DE);

    let mut description = describe::describe(rgb, &basic.name);
    if let Some(alt) = &basic_alternative {
        description.push_str(", bordering on ");
        description.push_str(&alt.name);
    }

    let lch = rgb.to_oklch();
    ColorInfo {
        hex: rgb.to_hex(),
        rgb,
        oklch: [lch.l as f32, lch.c as f32, lch.h as f32],
        descriptive: names.descriptive.nearest(lab),
        precise: names.precise.nearest(lab),
        description,
        basic,
        basic_alternative,
    }
}

/// Colour details for a vision profile.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ColorDetails {
    pub info: ColorInfo,
    /// Real-world objects.
    pub references: Vec<String>,
    /// Moods first, then industries and uses.
    pub associations: Vec<String>,
    /// Common pairings, as hex.
    pub pairings: Vec<String>,
    /// Which reference entry these came from ("mint", "similar to mint", "greens in general").
    pub reference_source: references::ReferenceSource,
    /// Named colours that look alike with this profile (see [`check::confused_with`]).
    pub confused_with: Vec<ColorName>,
}

pub fn color_details(rgb: Rgb, profile: simulate::VisionProfile) -> ColorDetails {
    let info = color_info(rgb);
    let (source, entry) = references::lookup(rgb, &info.basic.name);
    ColorDetails {
        references: entry.references.clone(),
        associations: entry
            .associations
            .iter()
            .chain(&entry.uses)
            .cloned()
            .collect(),
        pairings: entry.pairings.clone(),
        reference_source: source,
        confused_with: check::confused_with(rgb, profile),
        info,
    }
}

/// Fuzzy colour-name search across all bundled lists, best matches first.
pub fn search_color_by_name(query: &str, limit: usize) -> Vec<ColorName> {
    naming::Names::bundled().search(query, limit)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_hex_is_uppercase_and_zero_padded() {
        assert_eq!(Rgb::new(0x87, 0xCE, 0xEB).to_hex(), "#87CEEB");
        assert_eq!(Rgb::new(0, 0, 0).to_hex(), "#000000");
        assert_eq!(Rgb::new(255, 0, 10).to_hex(), "#FF000A");
    }
}
