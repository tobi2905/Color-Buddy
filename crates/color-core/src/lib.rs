//! Pure color logic for Color-Buddy.
//!
//! This crate has no Tauri dependency so it can be tested on its own. See `SPEC.md`.

pub mod convert;
pub mod delta_e;
pub mod describe;
pub mod naming;

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

/// Everything we know about a single colour.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColorInfo {
    pub hex: String,
    pub rgb: Rgb,
    /// OKLCH: L (0–1), C, h (degrees).
    pub oklch: [f32; 3],
    /// e.g. "blue"
    pub basic: ColorName,
    /// e.g. "sky blue" (CSS named colours)
    pub descriptive: ColorName,
    /// e.g. "light sky blue" (XKCD colour survey)
    pub precise: ColorName,
    /// e.g. "light, slightly desaturated, cool blue"
    pub description: String,
}

/// Names, OKLCH values and description for a colour.
pub fn color_info(rgb: Rgb) -> ColorInfo {
    let (basic, descriptive, precise) = naming::Names::bundled().name(rgb);
    let lch = rgb.to_oklch();
    ColorInfo {
        hex: rgb.to_hex(),
        rgb,
        oklch: [lch.l as f32, lch.c as f32, lch.h as f32],
        description: describe::describe(rgb, &basic.name),
        basic,
        descriptive,
        precise,
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
