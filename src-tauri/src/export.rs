//! Palette export: CSS variables, JSON, a Tailwind config snippet, and plain-text clipboard
//! formats. Names, RGB values and the check summary are computed from the colours, so exports
//! always match what the app shows.

use serde::{Deserialize, Serialize};
use serde_json::json;

use color_core::check::{PairVerdict, check_palette};
use color_core::palette::{Role, Scheme};
use color_core::simulate::VisionType;
use color_core::{ColorInfo, Rgb, color_info};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Format {
    Css,
    Json,
    Tailwind,
    /// Role, name, hex and RGB, one colour per line.
    TextDetailed,
    /// Hex values, one per line.
    TextHex,
    /// Names, one per line.
    TextNames,
    /// Name and hex, one colour per line.
    TextNamesHex,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NameLevel {
    Basic,
    #[default]
    Descriptive,
    Precise,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportColor {
    pub hex: String,
    pub role: Role,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportPalette {
    pub name: String,
    pub scheme: Scheme,
    pub base_hex: String,
    pub colors: Vec<ExportColor>,
}

struct Resolved {
    key: String,
    role: Role,
    rgb: Rgb,
    info: ColorInfo,
}

fn role_word(role: Role) -> &'static str {
    match role {
        Role::Background => "background",
        Role::Surface => "surface",
        Role::Primary => "primary",
        Role::Accent => "accent",
        Role::Text => "text",
    }
}

fn scheme_word(scheme: Scheme) -> &'static str {
    match scheme {
        Scheme::Complementary => "complementary",
        Scheme::Analogous => "analogous",
        Scheme::Triadic => "triadic",
        Scheme::SplitComplementary => "split-complementary",
        Scheme::Monochrome => "monochrome",
    }
}

fn vision_word(v: VisionType) -> &'static str {
    match v {
        VisionType::Normal => "normal",
        VisionType::Protan => "protan",
        VisionType::Deutan => "deutan",
        VisionType::Tritan => "tritan",
        VisionType::Achromat => "achromat",
    }
}

fn capitalised(s: &str) -> String {
    let mut c = s.chars();
    c.next()
        .map(|f| f.to_uppercase().chain(c).collect())
        .unwrap_or_default()
}

/// Lowercase letters, digits and single hyphens; "palette" when nothing is left.
pub fn slug(name: &str) -> String {
    let mut out = String::new();
    for ch in name.trim().to_lowercase().chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch);
        } else if !out.ends_with('-') && !out.is_empty() {
            out.push('-');
        }
    }
    let out = out.trim_end_matches('-').to_string();
    if out.is_empty() {
        "palette".into()
    } else {
        out
    }
}

/// Unique keys per role: background, surface, primary, accent, accent-2, text.
fn resolve(p: &ExportPalette) -> Result<Vec<Resolved>, String> {
    let mut seen: Vec<String> = Vec::new();
    p.colors
        .iter()
        .map(|c| {
            let rgb =
                Rgb::from_hex(&c.hex).ok_or_else(|| format!("not a hex colour: {}", c.hex))?;
            let base = role_word(c.role);
            let n = seen.iter().filter(|k| k.as_str() == base).count();
            seen.push(base.to_string());
            let key = if n == 0 {
                base.to_string()
            } else {
                format!("{base}-{}", n + 1)
            };
            Ok(Resolved {
                key,
                role: c.role,
                rgb,
                info: color_info(rgb),
            })
        })
        .collect()
}

fn name_of(info: &ColorInfo, level: NameLevel) -> &str {
    match level {
        NameLevel::Basic => &info.basic.name,
        NameLevel::Descriptive => &info.descriptive.name,
        NameLevel::Precise => &info.precise.name,
    }
}

pub fn export(p: &ExportPalette, format: Format, level: NameLevel) -> Result<String, String> {
    if p.colors.is_empty() {
        return Err("The palette has no colours to export.".into());
    }
    let colors = resolve(p)?;
    let lines = |f: &dyn Fn(&Resolved) -> String| -> String {
        colors.iter().map(f).collect::<Vec<_>>().join("\n") + "\n"
    };
    Ok(match format {
        Format::TextHex => lines(&|c| c.info.hex.clone()),
        Format::TextNames => lines(&|c| name_of(&c.info, level).to_string()),
        Format::TextNamesHex => lines(&|c| format!("{} {}", name_of(&c.info, level), c.info.hex)),
        Format::TextDetailed => lines(&|c| {
            format!(
                "{}: {}, {}, rgb({}, {}, {})",
                capitalised(&c.key.replace('-', " ")),
                name_of(&c.info, level),
                c.info.hex,
                c.rgb.r,
                c.rgb.g,
                c.rgb.b
            )
        }),
        Format::Css => {
            let mut out = format!(
                "/* {}: {} palette from {} (Color-Buddy) */\n:root {{\n",
                p.name.replace("*/", "* /"),
                scheme_word(p.scheme),
                p.base_hex
            );
            for c in &colors {
                out += &format!(
                    "  --color-{}: {}; /* {} */\n",
                    c.key,
                    c.info.hex,
                    name_of(&c.info, level).replace("*/", "* /")
                );
            }
            out + "}\n"
        }
        Format::Tailwind => {
            let mut out = format!(
                "// {}: add to tailwind.config.js (Tailwind v3, or v4 via @config).\n\
                 export default {{\n  theme: {{\n    extend: {{\n      colors: {{\n        \"{}\": {{\n",
                p.name.replace('\n', " "),
                slug(&p.name)
            );
            for c in &colors {
                out += &format!(
                    "          \"{}\": \"{}\", // {}\n",
                    c.key,
                    c.info.hex,
                    name_of(&c.info, level)
                );
            }
            out + "        },\n      },\n    },\n  },\n};\n"
        }
        Format::Json => {
            let rgbs: Vec<Rgb> = colors.iter().map(|c| c.rgb).collect();
            let roles: Vec<Role> = colors.iter().map(|c| c.role).collect();
            let checks = check_palette(&rgbs, Some(&roles));
            let by_vision: serde_json::Map<String, serde_json::Value> = VisionType::ALL
                .iter()
                .enumerate()
                .map(|(i, v)| {
                    let counted = checks.iter().filter(|c| !c.subtle);
                    let count = |verdict: PairVerdict| {
                        counted
                            .clone()
                            .filter(|c| c.results[i].2 == verdict)
                            .count()
                    };
                    (
                        vision_word(*v).to_string(),
                        json!({
                            "hardToTell": count(PairVerdict::HardToTell),
                            "caution": count(PairVerdict::Caution),
                        }),
                    )
                })
                .collect();
            let problems: Vec<_> = checks
                .iter()
                .filter(|c| !c.subtle)
                .flat_map(|c| {
                    c.results
                        .iter()
                        .filter(|r| r.2 != PairVerdict::Distinct)
                        .map(|r| {
                            json!({
                                "pair": [colors[c.a].key, colors[c.b].key],
                                "vision": vision_word(r.0),
                                "deltaE": ((r.1 as f64) * 10.0).round() / 10.0,
                                "verdict": r.2,
                            })
                        })
                        .collect::<Vec<_>>()
                })
                .collect();
            let doc = json!({
                "name": p.name,
                "scheme": p.scheme,
                "baseHex": p.base_hex,
                "colors": colors.iter().map(|c| json!({
                    "key": c.key,
                    "role": c.role,
                    "hex": c.info.hex,
                    "rgb": [c.rgb.r, c.rgb.g, c.rgb.b],
                    "names": {
                        "basic": c.info.basic.name,
                        "descriptive": c.info.descriptive.name,
                        "precise": c.info.precise.name,
                    },
                    "description": c.info.description,
                })).collect::<Vec<_>>(),
                "checks": {
                    "note": "Deficiencies simulated at full severity; background/surface tints are not counted.",
                    "byVision": by_vision,
                    "problems": problems,
                },
                "generator": "Color-Buddy",
            });
            serde_json::to_string_pretty(&doc).expect("serialisable") + "\n"
        }
    })
}

/// File extension for "Save as file…".
pub fn extension(format: Format) -> &'static str {
    match format {
        Format::Css => "css",
        Format::Json => "json",
        Format::Tailwind => "js",
        _ => "txt",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The sample palette for checkpoint 7 (sky blue, complementary). Outputs are compared with
    /// the committed files in `tests/fixtures/exports/`, which the frontend tests validate.
    pub(crate) fn sample() -> ExportPalette {
        let colors = color_core::palette::generate(
            Rgb::from_hex("#87CEEB").unwrap(),
            Scheme::SplitComplementary,
        )
        .colors
        .into_iter()
        .map(|c| ExportColor {
            hex: c.hex,
            role: c.role,
        })
        .collect();
        ExportPalette {
            name: "Sky afternoon".into(),
            scheme: Scheme::SplitComplementary,
            base_hex: "#87CEEB".into(),
            colors,
        }
    }

    const FORMATS: [(Format, &str); 7] = [
        (Format::Css, "sky-afternoon.css"),
        (Format::Json, "sky-afternoon.json"),
        (Format::Tailwind, "sky-afternoon.tailwind.js"),
        (Format::TextDetailed, "sky-afternoon.detailed.txt"),
        (Format::TextHex, "sky-afternoon.hex.txt"),
        (Format::TextNames, "sky-afternoon.names.txt"),
        (Format::TextNamesHex, "sky-afternoon.names-hex.txt"),
    ];

    fn fixture_dir() -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/exports")
    }

    /// `UPDATE_EXPORT_FIXTURES=1 cargo test -p color-buddy export` rewrites the fixtures.
    #[test]
    fn exports_match_committed_samples() {
        let update = std::env::var("UPDATE_EXPORT_FIXTURES").as_deref() == Ok("1");
        for (format, file) in FORMATS {
            let out = export(&sample(), format, NameLevel::Descriptive).unwrap();
            let path = fixture_dir().join(file);
            if update {
                std::fs::create_dir_all(fixture_dir()).unwrap();
                std::fs::write(&path, &out).unwrap();
            }
            let want = std::fs::read_to_string(&path).unwrap_or_default();
            assert_eq!(
                out, want,
                "{file} differs (rerun with UPDATE_EXPORT_FIXTURES=1)"
            );
        }
    }

    #[test]
    fn text_formats_match_their_descriptions() {
        let p = sample();
        let n = p.colors.len();
        let hex = export(&p, Format::TextHex, NameLevel::Descriptive).unwrap();
        assert_eq!(hex.lines().count(), n);
        assert!(hex.lines().all(|l| l.len() == 7 && l.starts_with('#')));

        let names = export(&p, Format::TextNames, NameLevel::Descriptive).unwrap();
        assert_eq!(names.lines().count(), n);
        assert!(names.lines().all(|l| !l.contains('#')));

        let names_hex = export(&p, Format::TextNamesHex, NameLevel::Descriptive).unwrap();
        for (line, h) in names_hex.lines().zip(hex.lines()) {
            assert!(line.ends_with(&format!(" {h}")), "{line}");
        }

        let detailed = export(&p, Format::TextDetailed, NameLevel::Descriptive).unwrap();
        let first = detailed.lines().next().unwrap();
        assert!(first.starts_with("Background: "), "{first}");
        assert!(first.contains(", #") && first.contains(", rgb("), "{first}");
        assert!(detailed.contains("Accent 2: "), "{detailed}");
    }

    #[test]
    fn name_level_changes_the_names() {
        let p = sample();
        let basic = export(&p, Format::TextNames, NameLevel::Basic).unwrap();
        assert!(basic.lines().all(|l| {
            [
                "red", "orange", "yellow", "green", "blue", "purple", "pink", "brown", "black",
                "white", "grey",
            ]
            .contains(&l)
        }));
    }

    #[test]
    fn json_is_valid_and_summarises_checks() {
        let out = export(&sample(), Format::Json, NameLevel::Descriptive).unwrap();
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["colors"].as_array().unwrap().len(), sample().colors.len());
        assert_eq!(v["colors"][0]["key"], "background");
        for vision in ["normal", "protan", "deutan", "tritan", "achromat"] {
            assert!(v["checks"]["byVision"][vision]["hardToTell"].is_u64());
        }
    }

    #[test]
    fn slugs_and_comment_safety() {
        assert_eq!(slug("Sky afternoon!"), "sky-afternoon");
        assert_eq!(slug("  ¿¿  "), "palette");
        let mut p = sample();
        p.name = "evil */ body { color: red }".into();
        let css = export(&p, Format::Css, NameLevel::Descriptive).unwrap();
        assert_eq!(css.matches("*/").count(), sample().colors.len() + 1);
    }

    #[test]
    fn rejects_empty_or_invalid_palettes() {
        let mut p = sample();
        p.colors[0].hex = "#nope".into();
        assert!(export(&p, Format::Css, NameLevel::Descriptive).is_err());
        p.colors.clear();
        assert!(export(&p, Format::Css, NameLevel::Descriptive).is_err());
    }
}
