//! Conversions and CIEDE2000 against published / independent reference values.

use color_core::Rgb;
use color_core::convert::Lab;
use color_core::delta_e::ciede2000;
use serde::Deserialize;

#[derive(Deserialize)]
struct Fixture {
    colors: Vec<Reference>,
}

#[derive(Deserialize)]
struct Reference {
    hex: String,
    linear: [f64; 3],
    xyz: [f64; 3],
    lab: [f64; 3],
    oklab: [f64; 3],
    oklch: [f64; 3],
}

fn assert_close(what: &str, hex: &str, got: [f64; 3], want: [f64; 3], tol: f64) {
    for i in 0..3 {
        assert!(
            (got[i] - want[i]).abs() <= tol,
            "{hex} {what}[{i}]: got {:.6}, want {:.6} (tol {tol})",
            got[i],
            want[i]
        );
    }
}

#[test]
fn conversions_match_colour_science_within_0_01() {
    let fixture: Fixture =
        serde_json::from_str(include_str!("fixtures/reference_conversions.json")).unwrap();
    for r in fixture.colors {
        let rgb = Rgb::from_hex(&r.hex).unwrap();
        let lin = rgb.to_linear();
        assert_close("linear", &r.hex, [lin.r, lin.g, lin.b], r.linear, 1e-4);
        let xyz = rgb.to_xyz();
        assert_close("xyz", &r.hex, [xyz.x, xyz.y, xyz.z], r.xyz, 1e-3);
        let lab = rgb.to_lab();
        assert_close("lab", &r.hex, [lab.l, lab.a, lab.b], r.lab, 0.01);
        let ok = rgb.to_oklab();
        assert_close("oklab", &r.hex, [ok.l, ok.a, ok.b], r.oklab, 1e-3);
        let lch = rgb.to_oklch();
        assert_close(
            "oklch",
            &r.hex,
            [lch.l, lch.c, 0.0],
            [r.oklch[0], r.oklch[1], 0.0],
            0.01,
        );
        // Hue is in degrees and undefined for neutral colours. The OKLab matrix variants
        // (direct from linear sRGB vs via XYZ) differ by up to ~0.02°.
        if r.oklch[1] >= 1e-3 {
            assert!(
                (lch.h - r.oklch[2]).abs() <= 0.05,
                "{} hue: got {}, want {}",
                r.hex,
                lch.h,
                r.oklch[2]
            );
        }
    }
}

/// Published values: CSS Color 4 (sample code, §18) and Björn Ottosson's OKLab post.
#[test]
fn conversions_match_published_values() {
    // sRGB red -> CIELAB D65 (Bruce Lindbloom calculator, D65, sRGB).
    let lab = Rgb::new(255, 0, 0).to_lab();
    assert_close(
        "lab",
        "#FF0000",
        [lab.l, lab.a, lab.b],
        [53.2408, 80.0925, 67.2032],
        0.01,
    );
    // White -> OKLab (1, 0, 0).
    let ok = Rgb::new(255, 255, 255).to_oklab();
    assert_close(
        "oklab",
        "#FFFFFF",
        [ok.l, ok.a, ok.b],
        [1.0, 0.0, 0.0],
        1e-3,
    );
    // sRGB red -> OKLCH (CSS Color 4: oklch(62.8% 0.2577 29.23)).
    let lch = Rgb::new(255, 0, 0).to_oklch();
    assert_close(
        "oklch",
        "#FF0000",
        [lch.l, lch.c, lch.h],
        [0.628, 0.2577, 29.23],
        0.01,
    );
}

#[test]
fn ciede2000_matches_sharma_2005_test_data() {
    let data = include_str!("fixtures/ciede2000_sharma.txt");
    let mut count = 0;
    for line in data
        .lines()
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
    {
        let v: Vec<f64> = line
            .split_whitespace()
            .map(|x| x.parse().unwrap())
            .collect();
        let a = Lab {
            l: v[0],
            a: v[1],
            b: v[2],
        };
        let b = Lab {
            l: v[3],
            a: v[4],
            b: v[5],
        };
        let got = ciede2000(a, b);
        assert!(
            (got - v[6]).abs() < 1e-4,
            "pair {}: got {got:.4}, want {:.4}",
            count + 1,
            v[6]
        );
        assert!(
            (ciede2000(b, a) - v[6]).abs() < 1e-4,
            "pair {} reversed",
            count + 1
        );
        count += 1;
    }
    assert_eq!(count, 34);
}
