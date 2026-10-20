//! Real-world references, associations, uses and pairings (`data/references.json`).
//!
//! Lookup falls back in three steps: the nearest entry when it is (nearly) the same colour,
//! then the nearest entry within [`SIMILAR_DE`] labelled "similar to …", then the entry for the
//! colour's basic family, so every colour gets at least family-level references.

use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

use crate::Rgb;
use crate::convert::Lab;
use crate::delta_e::ciede2000;

/// At or below this CIEDE2000 distance the entry is treated as the same colour.
pub const EXACT_DE: f64 = 2.0;
/// Up to this distance the nearest entry is used, labelled "similar to …".
pub const SIMILAR_DE: f64 = 10.0;

#[derive(Debug, Deserialize)]
struct RawEntry {
    hex: String,
    references: Vec<String>,
    associations: Vec<String>,
    uses: Vec<String>,
    pairings: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ReferenceEntry {
    pub name: String,
    pub hex: String,
    lab: Lab,
    pub references: Vec<String>,
    pub associations: Vec<String>,
    pub uses: Vec<String>,
    pub pairings: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ReferenceMatch {
    /// The entry is (nearly) this colour.
    Exact,
    /// The nearest entry, within [`SIMILAR_DE`].
    Similar,
    /// No entry close enough: the basic family's entry.
    Family,
}

/// Which entry the references came from, for labelling them in the UI.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceSource {
    pub name: String,
    pub hex: String,
    #[serde(rename = "match")]
    pub kind: ReferenceMatch,
    /// CIEDE2000 distance from the looked-up colour to the entry.
    pub delta_e: f32,
    /// e.g. "mint", "similar to mint", "blues in general"
    pub label: String,
}

/// All entries, parsed once from the bundled data.
pub fn entries() -> &'static [ReferenceEntry] {
    static ENTRIES: OnceLock<Vec<ReferenceEntry>> = OnceLock::new();
    ENTRIES.get_or_init(|| {
        let raw: serde_json::Map<String, serde_json::Value> =
            serde_json::from_str(include_str!("../../../data/references.json"))
                .expect("references.json");
        raw.into_iter()
            .map(|(name, value)| {
                let e: RawEntry = serde_json::from_value(value).expect("reference entry");
                let rgb = Rgb::from_hex(&e.hex).expect("reference hex");
                ReferenceEntry {
                    name,
                    hex: rgb.to_hex(),
                    lab: rgb.to_lab(),
                    references: e.references,
                    associations: e.associations,
                    uses: e.uses,
                    pairings: e.pairings,
                }
            })
            .collect()
    })
}

/// The entry for `rgb`, whose basic colour name is `basic`.
pub fn lookup(rgb: Rgb, basic: &str) -> (ReferenceSource, &'static ReferenceEntry) {
    let lab = rgb.to_lab();
    let (de, nearest) = entries()
        .iter()
        .map(|e| (ciede2000(lab, e.lab), e))
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .expect("references.json is empty");

    let (kind, entry, de, label) = if de <= EXACT_DE {
        (ReferenceMatch::Exact, nearest, de, nearest.name.clone())
    } else if de <= SIMILAR_DE {
        (
            ReferenceMatch::Similar,
            nearest,
            de,
            format!("similar to {}", nearest.name),
        )
    } else {
        let family = entries()
            .iter()
            .find(|e| e.name == basic)
            .unwrap_or_else(|| panic!("references.json has no family entry for {basic}"));
        let label = format!("{} colours in general", basic_plural(basic));
        (
            ReferenceMatch::Family,
            family,
            ciede2000(lab, family.lab),
            label,
        )
    };
    (
        ReferenceSource {
            name: entry.name.clone(),
            hex: entry.hex.clone(),
            kind,
            delta_e: de as f32,
            label,
        },
        entry,
    )
}

fn basic_plural(basic: &str) -> String {
    match basic {
        "black" | "white" => format!("{basic} and near-{basic}"),
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color_info;

    fn hex(h: &str) -> Rgb {
        Rgb::from_hex(h).unwrap()
    }

    #[test]
    fn bundled_data_has_at_least_200_entries_and_every_family() {
        assert!(entries().len() >= 200, "{}", entries().len());
        for term in crate::naming::Names::bundled()
            .basic
            .colors()
            .map(|(n, _)| n.name)
        {
            assert!(
                entries().iter().any(|e| e.name == term),
                "no family entry for {term}"
            );
        }
    }

    #[test]
    fn exact_similar_and_family_matches() {
        // The spec's example #98FF98 is within 2 ΔE of CSS "pale green" (#98FB98).
        let (src, entry) = lookup(hex("#98FF98"), "green");
        assert_eq!(src.kind, ReferenceMatch::Exact, "{src:?}");
        assert!(!entry.references.is_empty());

        let mint = entries().iter().find(|e| e.name == "mint").unwrap();
        let (src, _) = lookup(hex(&mint.hex), "green");
        assert_eq!(
            (src.kind, src.label.as_str()),
            (ReferenceMatch::Exact, "mint")
        );

        // A colour a few ΔE away from an entry.
        let (src, _) = lookup(hex("#85C1D9"), "blue");
        assert_eq!(src.kind, ReferenceMatch::Similar, "{src:?}");
        assert!(src.label.starts_with("similar to "));
    }

    #[test]
    fn every_entry_finds_itself() {
        for e in entries() {
            let rgb = hex(&e.hex);
            let (src, _) = lookup(rgb, &color_info(rgb).basic.name);
            // Near-duplicates (e.g. "mint" and "mint green") may resolve to each other.
            assert_eq!(src.kind, ReferenceMatch::Exact, "{} -> {src:?}", e.name);
        }
    }

    #[test]
    fn every_colour_gets_at_least_family_references() {
        let mut kinds = std::collections::HashMap::new();
        for r in (0..=255).step_by(15) {
            for g in (0..=255).step_by(15) {
                for b in (0..=255).step_by(15) {
                    let rgb = Rgb::new(r as u8, g as u8, b as u8);
                    let info = color_info(rgb);
                    let (src, entry) = lookup(rgb, &info.basic.name);
                    assert!(!entry.references.is_empty(), "{rgb:?}");
                    if src.kind == ReferenceMatch::Family {
                        assert_eq!(src.name, info.basic.name);
                    }
                    *kinds.entry(src.kind).or_insert(0) += 1;
                }
            }
        }
        // Most colours should find a specific (exact or similar) entry.
        let specific = kinds.get(&ReferenceMatch::Exact).unwrap_or(&0)
            + kinds.get(&ReferenceMatch::Similar).unwrap_or(&0);
        let total: i32 = kinds.values().sum();
        assert!(specific * 2 > total, "{kinds:?}");
    }
}
