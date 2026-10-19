//! Prints the Phase 2 review table (test colours with all names and descriptions) as Markdown.
//!
//!   cargo run -p color-core --example name_table

use color_core::{Rgb, color_info};

include!("../tests/fixtures/basic_name_colors.rs");

fn main() {
    println!("| Hex | CSS name | Basic | Descriptive (ΔE) | Precise (ΔE) | Description |");
    println!("| --- | --- | --- | --- | --- | --- |");
    for (css, hex, _) in TEST_COLORS {
        let i = color_info(Rgb::from_hex(hex).unwrap());
        println!(
            "| `{hex}` | {css} | {} | {} ({:.1}) | {} ({:.1}) | {} |",
            i.basic.name,
            i.descriptive.name,
            i.descriptive.delta_e,
            i.precise.name,
            i.precise.delta_e,
            i.description
        );
    }
}
