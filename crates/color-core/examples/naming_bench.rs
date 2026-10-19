//! Phase 2 benchmark: time to name a colour.
//!
//!   cargo run --release -p color-core --example naming_bench

use std::hint::black_box;
use std::time::Instant;

use color_core::naming::Names;
use color_core::{Rgb, color_info};

fn main() {
    let names = Names::bundled(); // parse lists + build trees once, outside the timing
    let colors: Vec<Rgb> = (0..20_000u32)
        .map(|i| {
            let v = i.wrapping_mul(2_654_435_761);
            Rgb::new(v as u8, (v >> 8) as u8, (v >> 16) as u8)
        })
        .collect();

    let per_us = |f: &dyn Fn(Rgb)| {
        let t = Instant::now();
        for &c in &colors {
            f(c);
        }
        t.elapsed().as_secs_f64() * 1e6 / colors.len() as f64
    };

    let lab = |c: Rgb| c.to_lab();
    println!(
        "basic (exhaustive, {} anchors): {:.1} µs",
        names.basic.len(),
        per_us(&|c| {
            black_box(names.basic.nearest(lab(c)));
        })
    );
    println!(
        "descriptive (k-d tree, {} names): {:.1} µs",
        names.descriptive.len(),
        per_us(&|c| {
            black_box(names.descriptive.nearest(lab(c)));
        })
    );
    println!(
        "precise (k-d tree, {} names): {:.1} µs",
        names.precise.len(),
        per_us(&|c| {
            black_box(names.precise.nearest(lab(c)));
        })
    );
    let full = per_us(&|c| {
        black_box(color_info(c));
    });
    println!(
        "full ColorInfo (3 names + description): {full:.1} µs = {:.3} ms",
        full / 1000.0
    );
}
