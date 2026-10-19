//! Colour space conversions: sRGB ↔ linear RGB ↔ XYZ (D65) ↔ CIELAB, and ↔ OKLab / OKLCH.
//!
//! The maths is done by the `palette` crate in `f64`; this module exposes small plain structs
//! so the rest of the crate (and its API) doesn't depend on palette's generic types.
//! Conversions are unclamped: palette's `from_color` would silently clamp out-of-gamut results.

use palette::convert::FromColorUnclamped;
use palette::white_point::D65;
use palette::{Lab as PLab, LinSrgb, Oklab as POklab, Oklch as POklch, Srgb, Xyz as PXyz};
use serde::{Deserialize, Serialize};

use crate::Rgb;

/// Linear-light sRGB, components nominally 0–1 (may be outside for out-of-gamut colours).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LinearRgb {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

/// CIE XYZ, D65 white point, Y of white = 1.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Xyz {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// CIELAB, D65 white point. L is 0–100.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Lab {
    pub l: f64,
    pub a: f64,
    pub b: f64,
}

/// OKLab. L is 0–1.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Oklab {
    pub l: f64,
    pub a: f64,
    pub b: f64,
}

/// OKLCH. L is 0–1, C ≥ 0, h in degrees 0–360 (0 for achromatic colours).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Oklch {
    pub l: f64,
    pub c: f64,
    pub h: f64,
}

impl Rgb {
    /// Parse `#RRGGBB`, `RRGGBB`, `#RGB` or `RGB` (case-insensitive, surrounding whitespace ignored).
    pub fn from_hex(hex: &str) -> Option<Self> {
        let h = hex.trim();
        let h = h.strip_prefix('#').unwrap_or(h);
        if !h.chars().all(|c| c.is_ascii_hexdigit()) {
            return None;
        }
        let expand = |c: u8| (c as char).to_digit(16).map(|v| (v * 17) as u8);
        match h.len() {
            6 => {
                let v = u32::from_str_radix(h, 16).ok()?;
                Some(Self::new((v >> 16) as u8, (v >> 8) as u8, v as u8))
            }
            3 => {
                let b = h.as_bytes();
                Some(Self::new(expand(b[0])?, expand(b[1])?, expand(b[2])?))
            }
            _ => None,
        }
    }

    pub fn to_linear(self) -> LinearRgb {
        let c: LinSrgb<f64> = Srgb::new(self.r, self.g, self.b)
            .into_format::<f64>()
            .into_linear();
        LinearRgb {
            r: c.red,
            g: c.green,
            b: c.blue,
        }
    }

    pub fn to_xyz(self) -> Xyz {
        self.to_linear().to_xyz()
    }

    pub fn to_lab(self) -> Lab {
        self.to_xyz().to_lab()
    }

    pub fn to_oklab(self) -> Oklab {
        self.to_linear().to_oklab()
    }

    pub fn to_oklch(self) -> Oklch {
        self.to_oklab().to_oklch()
    }
}

impl LinearRgb {
    pub fn to_xyz(self) -> Xyz {
        let c = PXyz::<D65, f64>::from_color_unclamped(self.palette());
        Xyz {
            x: c.x,
            y: c.y,
            z: c.z,
        }
    }

    pub fn to_oklab(self) -> Oklab {
        let c = POklab::<f64>::from_color_unclamped(self.palette());
        Oklab {
            l: c.l,
            a: c.a,
            b: c.b,
        }
    }

    /// Whether every component is within 0–1 (with a small tolerance for rounding).
    pub fn in_gamut(self) -> bool {
        const EPS: f64 = 1e-6;
        [self.r, self.g, self.b]
            .iter()
            .all(|v| (-EPS..=1.0 + EPS).contains(v))
    }

    /// Gamma-encode and round to 8 bits, clamping out-of-gamut components.
    pub fn to_rgb(self) -> Rgb {
        let c: Srgb<f64> = Srgb::from_linear(self.palette());
        let q = |v: f64| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
        Rgb::new(q(c.red), q(c.green), q(c.blue))
    }

    fn palette(self) -> LinSrgb<f64> {
        LinSrgb::new(self.r, self.g, self.b)
    }
}

impl Xyz {
    pub fn to_linear(self) -> LinearRgb {
        let c = LinSrgb::<f64>::from_color_unclamped(PXyz::<D65, f64>::new(self.x, self.y, self.z));
        LinearRgb {
            r: c.red,
            g: c.green,
            b: c.blue,
        }
    }

    pub fn to_lab(self) -> Lab {
        let c =
            PLab::<D65, f64>::from_color_unclamped(PXyz::<D65, f64>::new(self.x, self.y, self.z));
        Lab {
            l: c.l,
            a: c.a,
            b: c.b,
        }
    }
}

impl Lab {
    pub fn to_xyz(self) -> Xyz {
        let c =
            PXyz::<D65, f64>::from_color_unclamped(PLab::<D65, f64>::new(self.l, self.a, self.b));
        Xyz {
            x: c.x,
            y: c.y,
            z: c.z,
        }
    }

    pub fn to_rgb(self) -> Rgb {
        self.to_xyz().to_linear().to_rgb()
    }
}

impl Oklab {
    pub fn to_linear(self) -> LinearRgb {
        let c = LinSrgb::<f64>::from_color_unclamped(POklab::new(self.l, self.a, self.b));
        LinearRgb {
            r: c.red,
            g: c.green,
            b: c.blue,
        }
    }

    pub fn to_oklch(self) -> Oklch {
        let c = POklch::<f64>::from_color_unclamped(POklab::new(self.l, self.a, self.b));
        let h = c.hue.into_positive_degrees();
        Oklch {
            l: c.l,
            c: c.chroma,
            // Hue is meaningless for (near-)neutral colours; report 0 instead of noise.
            h: if c.chroma < 1e-6 { 0.0 } else { h },
        }
    }

    pub fn to_rgb(self) -> Rgb {
        self.to_linear().to_rgb()
    }
}

impl Oklch {
    pub fn to_oklab(self) -> Oklab {
        let c = POklab::<f64>::from_color_unclamped(POklch::new(self.l, self.c, self.h));
        Oklab {
            l: c.l,
            a: c.a,
            b: c.b,
        }
    }

    pub fn to_linear(self) -> LinearRgb {
        self.to_oklab().to_linear()
    }

    pub fn in_gamut(self) -> bool {
        self.to_linear().in_gamut()
    }

    pub fn to_rgb(self) -> Rgb {
        self.to_linear().to_rgb()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_hex_forms() {
        assert_eq!(Rgb::from_hex("#87CEEB"), Some(Rgb::new(0x87, 0xCE, 0xEB)));
        assert_eq!(Rgb::from_hex("87ceeb"), Some(Rgb::new(0x87, 0xCE, 0xEB)));
        assert_eq!(Rgb::from_hex(" #f0a "), Some(Rgb::new(0xFF, 0x00, 0xAA)));
        assert_eq!(Rgb::from_hex("#12345"), None);
        assert_eq!(Rgb::from_hex("#GGGGGG"), None);
        assert_eq!(Rgb::from_hex("#+12345"), None);
    }

    #[test]
    fn all_8_bit_greys_and_primaries_round_trip() {
        for v in 0..=255u8 {
            for rgb in [
                Rgb::new(v, v, v),
                Rgb::new(v, 0, 0),
                Rgb::new(0, v, 0),
                Rgb::new(0, 0, v),
            ] {
                assert_eq!(rgb.to_lab().to_rgb(), rgb);
                assert_eq!(rgb.to_oklab().to_rgb(), rgb);
                assert_eq!(rgb.to_oklch().to_rgb(), rgb);
            }
        }
    }

    #[test]
    fn neutral_hue_is_zero() {
        assert_eq!(Rgb::new(128, 128, 128).to_oklch().h, 0.0);
    }

    #[test]
    fn gamut_check() {
        assert!(Rgb::new(255, 0, 0).to_oklch().in_gamut());
        assert!(
            !Oklch {
                l: 0.9,
                c: 0.3,
                h: 260.0
            }
            .in_gamut()
        );
    }
}
