//! Machado (2009) simulation against DaltonLens at the published severity steps.

use color_core::Rgb;
use color_core::simulate::{VisionType, simulate};
use serde::Deserialize;

#[derive(Deserialize)]
struct Fixture {
    cases: Vec<Case>,
}

#[derive(Deserialize)]
struct Case {
    hex: String,
    deficiency: String,
    severity: f64,
    expected: String,
}

#[test]
fn matches_daltonlens_within_one_level() {
    let fixture: Fixture =
        serde_json::from_str(include_str!("fixtures/machado_reference.json")).unwrap();
    assert_eq!(fixture.cases.len(), 108);
    let mut failures = Vec::new();
    for c in fixture.cases {
        let vision = match c.deficiency.as_str() {
            "protan" => VisionType::Protan,
            "deutan" => VisionType::Deutan,
            "tritan" => VisionType::Tritan,
            other => panic!("unknown deficiency {other}"),
        };
        let got = simulate(Rgb::from_hex(&c.hex).unwrap(), vision, c.severity);
        let want = Rgb::from_hex(&c.expected).unwrap();
        // DaltonLens truncates to 8 bits where we round, so allow one level.
        let diff = got
            .r
            .abs_diff(want.r)
            .max(got.g.abs_diff(want.g))
            .max(got.b.abs_diff(want.b));
        if diff > 1 {
            failures.push(format!(
                "{} {} {}: got {}, want {}",
                c.hex,
                c.deficiency,
                c.severity,
                got.to_hex(),
                c.expected
            ));
        }
    }
    assert!(failures.is_empty(), "\n{}", failures.join("\n"));
}
