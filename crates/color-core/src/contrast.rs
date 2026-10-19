//! WCAG 2.x contrast ratio (APCA comes later).

use serde::{Deserialize, Serialize};

use crate::Rgb;

/// WCAG relative luminance (0 = black, 1 = white).
pub fn relative_luminance(rgb: Rgb) -> f64 {
    let c = rgb.to_linear();
    0.2126 * c.r + 0.7152 * c.g + 0.0722 * c.b
}

/// WCAG contrast ratio, 1.0 – 21.0, independent of argument order.
pub fn contrast_ratio(a: Rgb, b: Rgb) -> f64 {
    let (la, lb) = (relative_luminance(a), relative_luminance(b));
    let (hi, lo) = if la >= lb { (la, lb) } else { (lb, la) };
    (hi + 0.05) / (lo + 0.05)
}

/// Highest WCAG 2.x level a text/background pair meets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WcagLevel {
    /// ≥ 7:1 for normal text.
    Aaa,
    /// ≥ 4.5:1 for normal text.
    Aa,
    /// ≥ 3:1: large text (18pt, or 14pt bold) and UI components only.
    AaLarge,
    Fail,
}

pub fn wcag_level(ratio: f64) -> WcagLevel {
    // Compare at two decimals so 4.499… isn't rounded up into a pass.
    let r = (ratio * 100.0).floor() / 100.0;
    match r {
        r if r >= 7.0 => WcagLevel::Aaa,
        r if r >= 4.5 => WcagLevel::Aa,
        r if r >= 3.0 => WcagLevel::AaLarge,
        _ => WcagLevel::Fail,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ratio(a: &str, b: &str) -> f64 {
        contrast_ratio(Rgb::from_hex(a).unwrap(), Rgb::from_hex(b).unwrap())
    }

    #[test]
    fn black_on_white_is_21() {
        assert!((ratio("#000000", "#FFFFFF") - 21.0).abs() < 1e-9);
        assert!((ratio("#FFFFFF", "#000000") - 21.0).abs() < 1e-9);
        assert!((ratio("#777777", "#777777") - 1.0).abs() < 1e-9);
    }

    #[test]
    fn known_pairs_match_the_w3c_formula() {
        // Values from the WebAIM contrast checker (W3C formula).
        assert!((ratio("#777777", "#FFFFFF") - 4.48).abs() < 0.01);
        assert!((ratio("#767676", "#FFFFFF") - 4.54).abs() < 0.01);
        assert!((ratio("#0000FF", "#FFFFFF") - 8.59).abs() < 0.01);
        assert!((ratio("#FF0000", "#FFFFFF") - 4.00).abs() < 0.01);
    }

    #[test]
    fn levels() {
        assert_eq!(wcag_level(ratio("#777777", "#FFFFFF")), WcagLevel::AaLarge);
        assert_eq!(wcag_level(ratio("#767676", "#FFFFFF")), WcagLevel::Aa);
        assert_eq!(wcag_level(21.0), WcagLevel::Aaa);
        assert_eq!(wcag_level(1.5), WcagLevel::Fail);
    }
}
