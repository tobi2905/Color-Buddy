//! Phase 2 acceptance: test colours with unambiguous expected basic names.

use color_core::{Rgb, color_info};

include!("fixtures/basic_name_colors.rs");

#[test]
fn test_colors_get_expected_basic_names() {
    let mut wrong = Vec::new();
    for (css, hex, expected) in TEST_COLORS {
        let info = color_info(Rgb::from_hex(hex).unwrap());
        if info.basic.name != expected {
            wrong.push(format!(
                "{css} {hex}: got {} (anchor {}, dE {:.1}), want {expected}",
                info.basic.name, info.basic.hex, info.basic.delta_e
            ));
        }
    }
    assert!(wrong.is_empty(), "\n{}", wrong.join("\n"));
}
