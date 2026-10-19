//! Colour vision deficiency simulation.
//!
//! Protan, deutan and tritan use the Machado, Oliveira & Fernandes (2009) matrices, applied in
//! linear RGB and interpolated between the published severity steps (0.0, 0.1, …, 1.0).
//! Achromat blends towards relative luminance. Machado is known to be less accurate for tritan.

use serde::{Deserialize, Serialize};

use crate::Rgb;
use crate::convert::{Lab, LinearRgb};
use crate::machado_matrices::{DEUTAN, PROTAN, TRITAN};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum VisionType {
    Normal,
    Protan,
    Deutan,
    Tritan,
    Achromat,
}

impl VisionType {
    pub const ALL: [VisionType; 5] = [
        VisionType::Normal,
        VisionType::Protan,
        VisionType::Deutan,
        VisionType::Tritan,
        VisionType::Achromat,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct VisionProfile {
    pub vision: VisionType,
    /// 0.0 (none) – 1.0 (full dichromacy / monochromacy).
    pub severity: f32,
}

/// Simulate in linear RGB. The result may be slightly outside 0–1.
pub fn simulate_linear(c: LinearRgb, vision: VisionType, severity: f64) -> LinearRgb {
    let severity = severity.clamp(0.0, 1.0);
    let table = match vision {
        VisionType::Normal => return c,
        VisionType::Achromat => {
            let y = 0.2126 * c.r + 0.7152 * c.g + 0.0722 * c.b;
            let mix = |v: f64| v + (y - v) * severity;
            return LinearRgb {
                r: mix(c.r),
                g: mix(c.g),
                b: mix(c.b),
            };
        }
        VisionType::Protan => &PROTAN,
        VisionType::Deutan => &DEUTAN,
        VisionType::Tritan => &TRITAN,
    };
    let m = interpolate(table, severity);
    let v = [c.r, c.g, c.b];
    let row = |i: usize| m[i][0] * v[0] + m[i][1] * v[1] + m[i][2] * v[2];
    LinearRgb {
        r: row(0),
        g: row(1),
        b: row(2),
    }
}

/// Matrix for `severity`, linearly interpolated between the two nearest published steps.
fn interpolate(table: &[[[f64; 3]; 3]; 11], severity: f64) -> [[f64; 3]; 3] {
    let pos = severity * 10.0;
    let lo = (pos.floor() as usize).min(10);
    let hi = (lo + 1).min(10);
    let t = pos - lo as f64;
    let mut m = [[0.0; 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            m[i][j] = table[lo][i][j] * (1.0 - t) + table[hi][i][j] * t;
        }
    }
    m
}

fn clamp01(c: LinearRgb) -> LinearRgb {
    LinearRgb {
        r: c.r.clamp(0.0, 1.0),
        g: c.g.clamp(0.0, 1.0),
        b: c.b.clamp(0.0, 1.0),
    }
}

/// How `rgb` appears with this vision, as a displayable 8-bit colour.
pub fn simulate(rgb: Rgb, vision: VisionType, severity: f64) -> Rgb {
    simulate_linear(rgb.to_linear(), vision, severity).to_rgb()
}

/// How `rgb` appears with this vision, in CIELAB (clamped to sRGB, not quantised), for ΔE.
pub fn simulate_lab(rgb: Rgb, vision: VisionType, severity: f64) -> Lab {
    clamp01(simulate_linear(rgb.to_linear(), vision, severity))
        .to_xyz()
        .to_lab()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normal_and_zero_severity_are_identity() {
        let rgb = Rgb::new(0x87, 0xCE, 0xEB);
        for v in VisionType::ALL {
            assert_eq!(simulate(rgb, v, 0.0), rgb, "{v:?}");
        }
        assert_eq!(simulate(rgb, VisionType::Normal, 1.0), rgb);
    }

    #[test]
    fn greys_are_unchanged() {
        for v in VisionType::ALL {
            for g in [0u8, 64, 128, 200, 255] {
                let grey = Rgb::new(g, g, g);
                let out = simulate(grey, v, 1.0);
                for ch in [out.r, out.g, out.b] {
                    assert!(ch.abs_diff(g) <= 1, "{v:?} {grey:?} -> {out:?}");
                }
            }
        }
    }

    #[test]
    fn achromat_is_grey() {
        let out = simulate(Rgb::new(255, 0, 0), VisionType::Achromat, 1.0);
        assert!(out.r == out.g && out.g == out.b, "{out:?}");
    }

    #[test]
    fn interpolation_is_halfway_between_steps() {
        let m = interpolate(&PROTAN, 0.35);
        for i in 0..3 {
            for j in 0..3 {
                let want = (PROTAN[3][i][j] + PROTAN[4][i][j]) / 2.0;
                assert!((m[i][j] - want).abs() < 1e-12);
            }
        }
    }
}
