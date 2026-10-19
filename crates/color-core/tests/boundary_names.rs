//! Boundary colours against the Phase 2 sighted review (2026-10-19): the reviewer's basic name
//! must be the app's basic name or its alternative.

use color_core::{Rgb, color_info};

/// (CSS name, hex, reviewer's choice; "either" = both of the two closest terms are fine).
const REVIEW: [(&str, &str, &str); 21] = [
    ("coral", "#FF7F50", "orange"),
    ("salmon", "#FA8072", "pink"),
    ("tomato", "#FF6347", "orange"),
    ("violet", "#EE82EE", "purple"),
    ("orchid", "#DA70D6", "pink"),
    ("plum", "#DDA0DD", "pink"),
    ("lavender", "#E6E6FA", "white"),
    ("olive", "#808000", "green"),
    ("olive drab", "#6B8E23", "green"),
    ("khaki", "#F0E68C", "yellow"),
    ("teal", "#008080", "blue"),
    ("turquoise", "#40E0D0", "blue"),
    ("aquamarine", "#7FFFD4", "green"),
    ("cadet blue", "#5F9EA0", "blue"),
    ("tan", "#D2B48C", "brown"),
    ("sienna", "#A0522D", "either"),
    ("chocolate", "#D2691E", "either"),
    ("peru", "#CD853F", "orange"),
    ("maroon", "#800000", "red"),
    ("beige", "#F5F5DC", "white"),
    ("magenta", "#FF00FF", "pink"),
];

/// Known misses, kept visible rather than tuned away: violet is named pink (purple is 6.8 ΔE
/// further), far outside the near-tie range.
const KNOWN_MISSES: [&str; 1] = ["violet"];

#[test]
fn reviewer_choice_is_shown() {
    let mut wrong = Vec::new();
    for (css, hex, choice) in REVIEW {
        let info = color_info(Rgb::from_hex(hex).unwrap());
        let alt = info.basic_alternative.as_ref().map(|a| a.name.as_str());
        let ok = match choice {
            "either" => alt.is_some(),
            c => info.basic.name == c || alt == Some(c),
        };
        if ok == KNOWN_MISSES.contains(&css) {
            wrong.push(format!(
                "{css} {hex}: reviewer {choice}, app {} / {alt:?}{}",
                info.basic.name,
                if ok {
                    " (listed as a known miss but now correct)"
                } else {
                    ""
                }
            ));
        }
    }
    assert!(wrong.is_empty(), "\n{}", wrong.join("\n"));
}

#[test]
fn clear_colours_have_no_alternative() {
    for hex in [
        "#FF0000", "#0000FF", "#00FF00", "#FFFFFF", "#000000", "#87CEEB",
    ] {
        let info = color_info(Rgb::from_hex(hex).unwrap());
        assert_eq!(info.basic_alternative, None, "{hex}");
        assert!(!info.description.contains("bordering"), "{hex}");
    }
}

#[test]
fn boundary_description_names_both_terms() {
    let info = color_info(Rgb::from_hex("#A0522D").unwrap());
    assert_eq!(info.basic.name, "orange");
    assert_eq!(info.basic_alternative.unwrap().name, "brown");
    assert!(
        info.description.ends_with(", bordering on brown"),
        "{}",
        info.description
    );
}
