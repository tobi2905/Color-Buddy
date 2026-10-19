# Colorblind Design Assistant — Project Specification

A desktop app that helps colorblind designers pick, name and combine colors with confidence, without relying on their own color perception.

> **How to use this spec with Claude Code**
> - Work through the phases **in order, one phase at a time**.
> - At the end of each phase, **stop at the Review checkpoint**: summarize what was built, show how to run/test it, and wait for approval before starting the next phase.
> - Don't pull work forward from later phases. If something in a later phase blocks the current one, flag it instead.
> - If a requirement is unclear or seems wrong, ask instead of guessing.
> - Keep this file up to date: tick tasks as they are done and record decisions in the Decision log at the end.

---

## 1. Product summary

**Target users:** colorblind web designers, UI designers, illustrators and hobbyists.

**Core promise:** every color gets a name, a plain-language description and a real-world reference. Every palette comes with an explanation of why it works and a check that its colors stay distinguishable.

### MVP features

| Feature | Summary |
| --- | --- |
| Screen color picker | Global hotkey opens a magnifier loupe that follows the cursor. Samples a small area, shows hex, RGB, basic name, descriptive name and a plain-language description. Click freezes and copies. |
| Color vision profile | Onboarding: choose type (protan, deutan, tritan, achromat) and severity. Drives warnings and "confused with" hints. |
| Color details | Names at three levels, description, real-world references, associations, and colors commonly confused with it for the user's vision type. |
| Palette generator | Input from picker, hex/RGB, or name search ("sky blue"). Outputs complementary, analogous, triadic, split-complementary and monochrome palettes, each with roles and a one-line explanation. |
| Distinguishability + contrast check | Pass/warn grid for every color pair under normal vision and each colorblind type, plus WCAG contrast. |
| Export | CSS variables, JSON, Tailwind config. |

### Out of scope for the MVP

Hue highlighter overlay, screenshot/image analysis, full-screen simulation filter, Figma/ASE export, gradient generator, text-to-speech, mobile/camera mode, cloud sync, accounts, Linux builds.

### Design principles for the app itself

- **Never rely on color alone** in the app's own UI: always pair color with text, icons or patterns.
- Text contrast in the app UI meets WCAG AA at minimum.
- Plain language over jargon ("hard to tell apart" rather than "ΔE 6.2"; show numbers as secondary detail).
- Works fully offline. No telemetry.

---

## 2. Tech stack

| Layer | Choice |
| --- | --- |
| App shell | Tauri 2 |
| UI | React 18 + TypeScript + Vite |
| Styling | Tailwind CSS |
| UI state | Zustand |
| Core logic | Rust workspace crate `color-core` (pure, no Tauri dependency) |
| Color math | `palette` crate; CIEDE2000 implemented in `color-core` |
| Screen capture | `xcap` crate first; native fallback (Windows GDI/DXGI, macOS ScreenCaptureKit) if too slow or inaccurate |
| Cursor position | `device_query` (or `rdev`) |
| Global hotkey | `tauri-plugin-global-shortcut` |
| Clipboard | `tauri-plugin-clipboard-manager` |
| Storage | SQLite via `rusqlite` |
| Tests | `cargo test` (core), Vitest + React Testing Library (UI), Playwright (E2E, later) |
| Lint/format | `cargo fmt`, `cargo clippy -D warnings`, ESLint, Prettier |
| CI | GitHub Actions; `tauri-action` for release builds |

**Target platforms:** Windows 10/11 and macOS 13+.

---

## 3. Architecture

Three layers. All color math lives in Rust (`color-core`) so it is tested once and fast enough for live hover. The UI only displays results and calls commands.

```
┌──────────────────────── UI (React + TS, Tauri webview) ────────────────────────┐
│  Picker overlay  │  Color details  │  Palette builder  │  Profile + settings    │
└───────────────────────────────▲──────────────┬──────────────────────────────────┘
                 events (color-hovered)        │ commands (invoke)
┌───────────────────────────────┴──────────────▼──────────────────────────────────┐
│                         Core (Rust, Tauri backend: src-tauri)                   │
│  sampler  │  color-core crate  │  storage (SQLite)  │  exporter                 │
└─────▲───────────────▲───────────────────▲──────────────────────┬────────────────┘
      │               │                   │                      ▼
  OS screen APIs   bundled data       app.db (local)       clipboard / files
  + global hotkey  (names, references)
```

### Repository layout

```
/
├── SPEC.md                    # this file
├── CLAUDE.md                  # short working notes for Claude Code (created in Phase 0)
├── package.json
├── src/                       # React UI
│   ├── windows/
│   │   ├── main/              # main app window (details, palettes, settings)
│   │   └── picker/            # transparent always-on-top picker overlay
│   ├── components/
│   ├── store/                 # Zustand stores
│   ├── lib/                   # typed wrappers around Tauri commands
│   └── types/                 # TS types mirroring Rust structs
├── src-tauri/
│   ├── src/
│   │   ├── main.rs
│   │   ├── commands.rs        # #[tauri::command] functions
│   │   ├── sampler.rs         # cursor + screen sampling
│   │   ├── storage.rs         # SQLite
│   │   └── export.rs
│   └── tauri.conf.json
├── crates/
│   └── color-core/
│       ├── src/
│       │   ├── lib.rs
│       │   ├── convert.rs     # sRGB ↔ linear ↔ XYZ ↔ Lab ↔ OKLab/OKLCH
│       │   ├── delta_e.rs     # CIEDE2000
│       │   ├── naming.rs      # name lookup (k-d tree)
│       │   ├── describe.rs    # plain-language description
│       │   ├── palette.rs     # palette schemes + roles
│       │   ├── simulate.rs    # color vision simulation
│       │   ├── contrast.rs    # WCAG (+ APCA later)
│       │   └── check.rs       # pairwise distinguishability
│       └── tests/
└── data/
    ├── names_basic.json
    ├── names_xkcd.json
    ├── names_css.json
    └── references.json
```

### Core data types (Rust, mirrored in TypeScript)

```rust
pub struct Rgb { pub r: u8, pub g: u8, pub b: u8 }

pub enum VisionType { Normal, Protan, Deutan, Tritan, Achromat }

pub struct VisionProfile { pub vision: VisionType, pub severity: f32 } // 0.0–1.0

pub struct ColorName { pub name: String, pub hex: String, pub delta_e: f32 }

pub struct ColorInfo {
    pub hex: String,
    pub rgb: Rgb,
    pub oklch: [f32; 3],          // L, C, h
    pub basic: ColorName,         // e.g. "blue"
    pub descriptive: ColorName,   // e.g. "sky blue"
    pub precise: ColorName,       // e.g. "light cornflower"
    pub description: String,      // e.g. "light, slightly desaturated, cool blue"
}

pub struct ColorDetails {
    pub info: ColorInfo,
    pub references: Vec<String>,  // real-world objects
    pub associations: Vec<String>,// moods, industries, uses
    pub pairings: Vec<String>,    // common pairings (hex or names)
    pub confused_with: Vec<ColorName>, // for the given profile
}

pub enum Scheme { Complementary, Analogous, Triadic, SplitComplementary, Monochrome }

pub enum Role { Background, Surface, Primary, Accent, Text }

pub struct PaletteColor { pub hex: String, pub role: Role, pub info: ColorInfo }

pub struct Palette {
    pub scheme: Scheme,
    pub base_hex: String,
    pub colors: Vec<PaletteColor>,
    pub explanation: String,      // why it works, one or two lines
}

pub enum PairVerdict { Distinct, Caution, HardToTell } // thresholds in Phase 3

pub struct PairCheck {
    pub a: usize, pub b: usize,
    pub results: Vec<(VisionType, f32, PairVerdict)>, // ΔE under each vision type
    pub contrast_ratio: f32,
}
```

### Tauri commands (contract)

| Command | Input | Output |
| --- | --- | --- |
| `get_color_info` | `hex` | `ColorInfo` |
| `get_color_details` | `hex`, `profile` | `ColorDetails` |
| `search_color_by_name` | `query` | `Vec<ColorName>` (best matches, fuzzy) |
| `generate_palettes` | `base_hex` | `Vec<Palette>` (one per scheme) |
| `check_palette` | `Vec<hex>` | `Vec<PairCheck>` |
| `start_picker` / `stop_picker` | — | — |
| `freeze_pick` | — | `ColorInfo` (also copies hex, adds to history) |
| `get_profile` / `set_profile` | `VisionProfile` | `VisionProfile` |
| `list_history` | `limit` | `Vec<HistoryEntry>` |
| `save_palette` / `list_palettes` / `delete_palette` | `Palette` / — / `id` | `SavedPalette` / `Vec<SavedPalette>` / — |
| `export_palette` | `id` or `Palette`, `format` (`css` \| `json` \| `tailwind`) | `String` |

**Event:** `color-hovered` → `{ x, y, info: ColorInfo, loupe: [[Rgb; N]; N] }`, emitted about 30 times per second while the picker is active.

---

## 4. Phases

Each phase lists its goal, tasks, acceptance criteria and a review checkpoint. Phases 1–3 correspond to "Prove sampling" and "Color core" in the roadmap; phases 4–7 build the MVP UI; phase 8 is beta.

---

### Phase 0 — Project scaffold

**Goal:** an empty but working Tauri + React + Rust workspace with tooling and CI.

**Tasks**
- [x] Create Tauri 2 app with React + TypeScript + Vite template.
- [x] Convert to a Cargo workspace; add empty `crates/color-core` library crate.
- [x] Add Tailwind, Zustand, ESLint, Prettier, Vitest.
- [x] Add `cargo fmt`, `cargo clippy -D warnings`, `cargo test`, `npm run lint`, `npm test` scripts.
- [x] GitHub Actions workflow: lint + test on Windows and macOS for every push.
- [x] Create `CLAUDE.md` with run/test commands and a pointer to this spec.
- [x] README with setup steps.

**Acceptance criteria**
- `npm run tauri dev` opens a window on Windows and macOS.
- All lint and test commands pass locally and in CI.

**🔍 Review checkpoint 0:** show the repo tree, the commands to run, and the CI result. Wait for approval.

---

### Phase 1 — Screen sampling spike (highest risk, do first)

**Goal:** prove that we can read the correct pixel colors under the cursor, fast enough, on both platforms.

**Tasks**
- [x] `sampler.rs`: get global cursor position in physical pixels on each monitor.
- [x] Capture an N×N area (N = 1, 3, 5, configurable) around the cursor.
- [x] Handle HiDPI and multi-monitor setups with different scale factors. *(Implemented per display; verified on a single 2× Retina display only. Multi-monitor still untested.)*
- [x] Convert captured pixels to sRGB; detect and log when the display profile isn't sRGB (no correction yet). *(macOS: the screen buffer is stored in the display profile, so captured pixels are converted back to sRGB with ColorSync. Windows: no correction, profile is logged.)*
- [x] Register a global hotkey (default `Ctrl+Shift+C` / `Cmd+Shift+C`) that starts/stops sampling.
- [x] Emit a temporary `color-hovered` event with hex only; show it in a bare debug window.
- [x] macOS: detect missing Screen Recording permission and surface a clear error.
- [x] Measure: latency from hotkey to first sample, and samples per second. *(macOS done; Windows pending, see TODO below.)*
- [ ] **TODO (deferred, no Windows machine yet):** verify on Windows, including a display at 150% scaling:
    - `cargo run --release --example sampler_bench` (timing per N) and `... -- watch` while hovering `tools/test-patches.png`.
    - `$env:VITE_SELFTEST="1"; npm run tauri dev` and check the `self-test` log lines (expect every patch exact at N = 1, 3, 5).
    - Press `Ctrl+Shift+C` in the app and read "Hotkey to first sample" in the debug window.

**Acceptance criteria**
- Hovering a known test image (pure #FF0000, #00FF00, #0000FF, #87CEEB patches) reports the exact hex on sRGB displays on both platforms, including on a scaled (e.g. 150%) display. On wide-gamut (e.g. Display P3) Macs, each channel is within 3 of the expected value; fully saturated colors at the gamut edge can't round-trip exactly through the 8-bit display buffer (see Decision log).
- At least 30 samples per second with N = 5.
- Hotkey to first sample in under 150 ms.

**🔍 Review checkpoint 1:** report measured numbers per platform, which capture approach was used (xcap vs native), and any platform issues. Wait for approval before building on this approach.

---

### Phase 2 — color-core: conversions, naming, description

**Goal:** given any hex, return correct `ColorInfo`.

**Tasks**
- [x] `convert.rs`: sRGB ↔ linear RGB ↔ XYZ (D65) ↔ CIELAB, and ↔ OKLab/OKLCH.
- [x] `delta_e.rs`: CIEDE2000.
- [x] Prepare `data/` name lists:
    - `names_basic.json`: 11 basic terms (red, orange, yellow, green, blue, purple, pink, brown, black, white, grey) with representative hex values.
    - `names_xkcd.json`: XKCD color survey names (CC0).
    - `names_css.json`: CSS named colors.
- [x] `naming.rs`: load lists, precompute Lab, build a k-d tree per list; nearest lookup by Lab, refined by CIEDE2000 over the k nearest candidates.
- [x] `describe.rs`: map OKLCH to words for lightness (very dark … very light), chroma (greyish … vivid), temperature (warm/cool/neutral), plus the basic hue name.
- [x] `search_color_by_name`: fuzzy match across all lists (e.g. "skyblue", "sky blue", "Sky-Blue" all match).
- [x] Expose `get_color_info` and `search_color_by_name` as Tauri commands.

**Acceptance criteria**
- Conversions match published reference values (e.g. Bruce Lindbloom / CSS Color 4 test values) within 0.01.
- CIEDE2000 passes the Sharma et al. (2005) test data set.
- Name lookup under 1 ms per color (benchmark).
- A list of 30 test colors with expected basic names is all correct; descriptive names reviewed by the user.

**🔍 Review checkpoint 2:** provide a table of 30 test colors with hex, basic, descriptive, precise name and description, so the user (who is colorblind) can sanity-check them with help from a non-colorblind person if needed. Wait for approval.

---

### Phase 3 — color-core: palettes, simulation, checks

**Goal:** generate palettes and judge whether their colors are distinguishable and readable.

**Tasks**
- [x] `palette.rs`: generate schemes in OKLCH, holding lightness and chroma where sensible:
    - Complementary (h + 180°), Analogous (h ± 30°), Triadic (h ± 120°), Split-complementary (h ± 150°), Monochrome (lightness steps only).
    - *Changed at checkpoint 3: accents step 0.2 (OKLCH L) lighter or darker than the primary instead of holding its lightness, so they stay apart for colourblind viewers.*
    - Gamut-map results back into sRGB (reduce chroma until in gamut).
    - Assign roles (background, surface, primary, accent, text) by lightness and chroma.
    - Write a one-line `explanation` per palette in plain language.
- [x] `simulate.rs`: Machado et al. (2009) matrices for protan, deutan, tritan with severity 0–1 (interpolate between the published severity steps); achromat = luminance only. Apply in linear RGB.
- [x] `check.rs`: for each color pair, ΔE2000 under normal vision and each simulated type. Initial thresholds (tune later):
    - ΔE ≥ 15 → `Distinct`
    - 8 ≤ ΔE < 15 → `Caution`
    - ΔE < 8 → `HardToTell`
    - *Tuned at checkpoint 3: these apply to normal vision; simulated deficiencies use 20 / 10. Background/surface pairs are marked `subtle` and not counted as warnings.*
- [x] `contrast.rs`: WCAG 2.x contrast ratio; flag text/background pairs below 4.5:1 (AA) and 7:1 (AAA).
- [x] "Confused with": simulate a color under the user's profile, return named colors within a small ΔE (start at 5) of the simulated result, excluding near-identical names.
- [x] Expose `generate_palettes`, `check_palette` and `get_color_details` (without references yet) as commands.

**Acceptance criteria**
- Simulation output matches reference images/values from the Machado paper or a trusted implementation (e.g. DaltonLens) within a small tolerance.
- Known confusable pairs are flagged (e.g. #FF0000 vs #008000 under deutan = `HardToTell` or `Caution`).
- WCAG ratios match the W3C formula on test pairs (black/white = 21:1).
- All generated colors are valid sRGB.

**🔍 Review checkpoint 3:** show generated palettes for 5 base colors with explanations and check grids (as a table or a small debug HTML page). Discuss whether thresholds feel right. Wait for approval.

---

### Phase 4 — Reference data

**Goal:** real-world references, associations and pairings for common colors.

**Tasks**
- [ ] Define `references.json` schema:
    ```json
    {
      "mint": {
        "hex": "#98FF98",
        "references": ["toothpaste", "mint ice cream", "medical scrubs"],
        "associations": ["fresh", "clean", "calm"],
        "uses": ["healthcare", "eco brands", "spring themes"],
        "pairings": ["#FFFFFF", "#2F4F4F", "#FFB6C1"]
      }
    }
    ```
- [ ] Write entries for the top 200 colors (prioritize basic hues and their common light/dark/muted variants).
- [ ] Fallback: when a color has no entry, use the nearest entry within a ΔE limit and label it "similar to <name>"; otherwise use the basic-name family entry.
- [ ] Validation script: schema check, valid hex, no duplicate keys.
- [ ] Complete `get_color_details` with references.

**Acceptance criteria**
- 200 entries pass validation.
- Every color returns at least a family-level reference.
- User has reviewed a sample of 30 entries.

**🔍 Review checkpoint 4:** share the schema, the list of covered colors, and 30 sample entries for review. Wait for approval.

---

### Phase 5 — UI: picker overlay

**Goal:** the everyday picker experience.

**Tasks**
- [ ] Separate transparent, always-on-top, click-through picker window that follows the cursor (offset so it never covers the sampled point).
- [ ] Loupe: magnified N×N grid with the center pixel outlined.
- [ ] Label: descriptive name (large), basic name, hex, and short description. Show ΔE-based confidence subtly ("closest match").
- [ ] Click or hotkey to freeze → copy hex to clipboard → toast "Copied #87CEEB (sky blue)" → add to history.
- [ ] `Esc` closes the picker; arrow keys nudge one pixel.
- [ ] Stays readable on any background (solid panel, high contrast).

**Acceptance criteria**
- Updates smoothly at 30+ fps with no visible lag.
- Works across monitors with different scaling.
- Copy + history work from the frozen state.

**🔍 Review checkpoint 5:** short demo instructions (or screen recording steps) for the user to try it. Wait for approval.

---

### Phase 6 — UI: main window (details, palettes, onboarding)

**Goal:** the main app window with all MVP screens.

**Tasks**
- [ ] Onboarding (first launch): explain the app, choose vision type and severity (with a "not sure" option defaulting to deutan, moderate, and a note that this isn't a medical test), set the hotkey, macOS permission step.
- [ ] Color input bar: hex, RGB, or name search with autocomplete; "pick from screen" button.
- [ ] Color details view: swatch, three name levels, description, references, associations, pairings, "confused with" list for the user's profile.
- [ ] Palette builder: scheme tabs, swatches with role and name labels, explanation, check grid (pairs × vision types) using icons + text (✓ distinct, ! caution, ✕ hard to tell), contrast badges.
- [ ] Edit a palette: replace a color, lock colors, regenerate the rest.
- [ ] History view.
- [ ] Settings: profile, hotkey, sample size, theme (light/dark), default name level.
- [ ] App UI itself never uses color as the only signal; passes an accessibility lint.

**Acceptance criteria**
- A first-time user can go from launch to an exported, checked palette in under 2 minutes.
- All screens usable with keyboard only.
- UI component tests cover input parsing, details rendering and check-grid rendering.

**🔍 Review checkpoint 6:** walkthrough of each screen. Wait for approval.

---

### Phase 7 — Storage and export

**Goal:** persist user data and export palettes.

**Tasks**
- [ ] `storage.rs` SQLite schema with migrations:
    - `profile(vision, severity, name_level)`
    - `history(id, hex, picked_at, source)` — capped at 500 entries
    - `palettes(id, name, created_at, base_hex, scheme)`
    - `palette_colors(palette_id, position, hex, role)`
    - `settings(key, value)`
- [ ] Save, rename, delete, list palettes; palette library view in the UI.
- [ ] `export.rs`:
    - CSS: `:root { --color-primary: #...; }` using role names.
    - JSON: palette with names, roles, hex, check summary.
    - Tailwind: `theme.extend.colors` snippet.
- [ ] Export UI: copy to clipboard or save file.

**Acceptance criteria**
- Data survives restarts; migrations run cleanly on a fresh install and on an existing DB.
- Export output is valid (CSS parses, JSON validates, Tailwind snippet loads in a sample config).

**🔍 Review checkpoint 7:** sample export files for one palette in each format. Wait for approval.

---

### Phase 8 — Polish, packaging and beta

**Goal:** a signed, installable beta tested by colorblind users.

**Tasks**
- [ ] App icon, tray icon with quick actions (pick, open app, quit), launch-at-login option.
- [ ] Error handling and friendly messages for all failure paths (permission missing, capture failure, DB error).
- [ ] Performance pass: startup under 1 s, memory under 150 MB idle.
- [ ] Release workflow with `tauri-action`: Windows installer and macOS DMG.
- [ ] Code signing: Apple Developer ID + notarization; Windows code-signing certificate (user to provide credentials; never commit them).
- [ ] In-app "report a wrong name" option (writes to a local file the user can send; no network).
- [ ] Beta with at least 5 colorblind testers; collect feedback on names, references and thresholds.

**Acceptance criteria**
- Installers install and run on clean Windows 11 and macOS machines without security warnings (once signing is set up).
- 5 testers complete the core flow; top issues fixed or logged.

**🔍 Review checkpoint 8:** beta feedback summary and a go/no-go for 1.0.

---

## 5. After 1.0 (backlog, not to be built yet)

- Hue highlighter overlay ("show me everything red on screen")
- Screenshot / image analysis with problem-pair flags
- Full-screen colorblind simulation filter
- Compare two colors with a verdict in words
- Figma, ASE and SCSS export
- Gradient generator with step distinguishability checks
- APCA contrast score
- Display color profile correction
- Text-to-speech for color names
- Linux support (X11, then xdg-desktop-portal on Wayland)

---

## 6. Open questions

- App name?
- Default hotkey: does `Ctrl/Cmd+Shift+C` clash with tools you use (it opens DevTools' element picker in browsers)?
- License for the app itself (open source or closed)?
- Should the "not sure" vision profile default to deutan, or should onboarding include a short self-check?

## 7. Decision log

| Date | Decision | Reason |
| --- | --- | --- |
| 2026-10-19 | Tauri 2 + React/TS + Rust core | Small binary, native screen access, fast UI iteration |
| 2026-10-19 | Machado (2009) for simulation, CIEDE2000 for distance, OKLCH for palettes | Severity support; perceptual accuracy |
| 2026-10-19 | Pinned React 18.3 (template defaults to 19) | Spec names React 18; revisit when moving to 19 |
| 2026-10-19 | Tailwind v4 via `@tailwindcss/vite` (no `tailwind.config.js`) | Current Tailwind major; config lives in CSS |
| 2026-10-19 | ESLint 9 (not 10) | `eslint-plugin-jsx-a11y` doesn't support ESLint 10 yet; a11y lint needed for Phase 6 |
| 2026-10-19 | Cargo workspace at repo root (`src-tauri`, `crates/color-core`); release profile at root | Profiles only apply from the workspace root |
| 2026-10-19 | Working app name "Color-Buddy", identifier `com.colorbuddy.app` | Placeholder until the app-name open question is settled |
| 2026-10-19 | Cursor position read directly from the OS (`CGEventGetLocation` / `GetCursorPos`), not `device_query` | `device_query` requires macOS Accessibility permission and truncates to whole points; Tauri's `cursor_position` uses the primary display's scale on every monitor |
| 2026-10-19 | Windows capture: `xcap` (GDI region capture, physical pixels) | Pixel-exact and fast enough; no colour management on the Windows desktop |
| 2026-10-19 | macOS capture: native CoreGraphics (`CGWindowListCreateImage`) + ColorSync conversion to sRGB, not `xcap` | xcap uses the same API but returns display-profile values (sRGB #FF0000 read as #EA3323 on P3) and whole-point regions. ~20 ms/capture (vsync-bound), ~44/s headroom |
| 2026-10-19 | ScreenCaptureKit evaluated and not adopted for now | Screenshot API ~45 ms per capture (< 30/s) and no accuracy gain (#00FF00 → #03FF00 SDR, #02FF00 HDR). Revisit if Apple removes `CGWindowListCreateImage` (deprecated) |
| 2026-10-19 | Acceptance relaxed to ±3 per channel on wide-gamut Macs | 8-bit P3 screen buffer loses precision for sRGB gamut-edge colours (#00FF00 stored as #75FB4C → #03FF00); approved by user |
| 2026-10-19 | Sampling loop capped at 30 Hz | Matches the `color-hovered` rate in the spec; capture capacity is higher |
| 2026-10-19 | Name levels: basic = 11 basic terms, descriptive = CSS named colours (148), precise = XKCD survey (922) | CSS names are familiar to web designers; XKCD is finer-grained. Approved by user |
| 2026-10-19 | Basic-term anchors = every XKCD survey name whose last word is the term (540 anchors, e.g. "reddish brown" → brown); searched exhaustively | Survey-derived rather than hand-picked; only the 11 plain terms + light/dark variants misnamed indigo and sienna. Exhaustive because white/black have only 2 anchors each (~0.13 ms) |
| 2026-10-19 | k-d tree in chroma-compressed Lab (a*, b* ÷ (1 + 0.045·C)), K = 64, then CIEDE2000 | Plain Lab with K = 8 picked a different name than exhaustive search for 7% of XKCD lookups; K = 64 differs for 0.05% (≤ 1.0 ΔE worse) at ~10 µs |
| 2026-10-19 | Removed 27 crude XKCD names (e.g. containing "puke", "poop"); split 4 glued names ("lightblue" → "light blue") | Names are shown to users |
| 2026-10-19 | Conversion references use the sRGB matrix derived from its primaries (CSS Color 4 / Lindbloom); OKLCH hue compared within 0.05° | The 4-decimal IEC matrix shifts CIELAB by ~0.02; OKLab matrix variants differ by ~0.02° in hue |
| 2026-10-19 | Description saturation = min(chroma relative to the sRGB gamut at that L/h, C / 0.15) | Relative chroma alone called pale pink "vivid" |
| 2026-10-19 | Basic-name test set avoids boundary colours (coral, violet, olive, teal, tan, sienna); sienna #A0522D is named orange | Boundary colours have no single correct basic name |
| 2026-10-19 | Sighted review: all 32 test colours fit; 21 boundary colours checked (results in `tests/boundary_names.rs`) | Required by checkpoint 2 |
| 2026-10-19 | Boundary colours show both basic terms, closer one first (`basicAlternative`, description ends ", bordering on X") when the second term is within 2.0 ΔE | User's call. Every review case of "either" or second-term preference had a gap ≤ 1.5 |
| 2026-10-19 | Known miss: violet #EE82EE is named pink (reviewer: purple, 6.8 ΔE further) | Not tuned away for one colour; revisit with beta feedback (Phase 8) |
| 2026-10-19 | "Confused with" = CSS-named colours that look alike under the user's profile (simulated ΔE < 5) but differ for typical vision (ΔE ≥ 15) and belong to another basic family | User's choice: a mix-up warning rather than a description of their perception |
| 2026-10-19 | Check grid uses full severity for every deficiency | User's choice: a passing palette works for all colourblind viewers; the profile drives "confused with" |
| 2026-10-19 | Machado matrices exported from DaltonLens (`tools/gen_machado.py`); severities between steps interpolated linearly | DaltonLens' own in-between interpolation uses an unscaled weight, so tests compare only at the published steps (108 cases, ±1 per channel) |
| 2026-10-19 | Palettes: background/surface/text are tints and a shade of the base hue (OKLCH L 0.98 / 0.94 / 0.25); primary is the base; accents rotate hue at the base's L (clamped 0.35–0.85) and C; near-grey bases get accent chroma 0.10; chroma reduced to fit sRGB | Text on background ≥ 7:1 for every base tested (216 bases × 5 schemes) |
| 2026-10-19 | PairCheck/ColorDetails and enums serialise as camelCase (`hardToTell`, `splitComplementary`, `contrastRatio`) | Matches the TypeScript mirror |
| 2026-10-19 | Thresholds: normal vision 15 / 8, simulated deficiencies 20 / 10 | User's choice at checkpoint 3. Red vs green under deutan measured 15.7 (lightness only); the margin flags it without adding noise for typical vision |
| 2026-10-19 | Background/surface pairs marked `subtle` and excluded from warnings (`check_palette` takes optional roles) | User's choice: a deliberate subtle tint was "hard to tell" in every palette |
| 2026-10-19 | Accents offset 0.2 OKLCH L from the primary (either side when both fit, else two steps one way; clamped 0.30–0.90) | User's choice. Across the 25 review palettes, hard-to-tell pairs fell: protan 12→3, deutan 13→1, tritan 7→0, achromat 78→19 |