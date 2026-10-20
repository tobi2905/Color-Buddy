# ColorBuddy — working notes for Claude Code

Read `SPEC.md` first. It is the source of truth: work one phase at a time, stop at each
Review checkpoint, tick tasks as they are done, and record decisions in its Decision log.

## Layout

- `src/` — React 18 + TS UI (Vite, Tailwind v4, Zustand). Entry: `src/windows/main/main.tsx`.
- `src-tauri/` — Tauri 2 backend (`color-buddy` crate).
- `crates/color-core/` — pure color logic, no Tauri dependency. All color math goes here.
- Root `Cargo.toml` is the Cargo workspace; build output goes to `/target`.

## Commands

| Task                                       | Command                                        |
| ------------------------------------------ | ---------------------------------------------- |
| Install                                    | `npm install`                                  |
| Run the app                                | `npm run tauri dev`                            |
| JS lint (ESLint + Prettier check)          | `npm run lint`                                 |
| Typecheck                                  | `npm run typecheck`                            |
| JS tests (Vitest)                          | `npm test`                                     |
| Rust lint (fmt check + clippy -D warnings) | `npm run lint:rust`                            |
| Rust tests                                 | `npm run test:rust` (`cargo test --workspace`) |
| Everything CI runs                         | `npm run build && npm run check`               |
| Auto-format JS + Rust                      | `npm run format`                               |

`cargo clippy`/`cargo test` on the workspace need `dist/` to exist (Tauri embeds it), so run
`npm run build` once first on a fresh checkout.

## Dev switches (Phase 1 sampler)

- `VITE_SELFTEST=1 npm run tauri dev`: samples the debug window's test patches on launch; results in the log (`self-test ...`).
- `COLOR_BUDDY_ON_TOP=1`: keeps the debug window above other windows.
- `COLOR_BUDDY_AUTOSTART=<seconds>`: opens the picker overlay for that long after launch and logs `sampling stopped: ... /s`.
- `VITE_SELFTEST_DELAY_MS`: when the self-test runs (default 1500), e.g. after `COLOR_BUDDY_AUTOSTART` opened the picker.
- `cargo run --release --example sampler_bench [-- watch]`: capture timing per N, or live hex under the cursor.
- macOS: the app that launches these (e.g. VS Code) needs Screen Recording permission.

## Color core (Phase 2)

- Name data in `data/` is generated: `python3 tools/gen_name_data.py <xkcd_rgb.txt> <css-color.html>` (sources in the script).
- Conversion test expectations: `tools/gen_reference_values.py` (needs `colour-science`).
- `cargo run --release -p color-core --example naming_bench`: lookup timing.
- `cargo run -p color-core --example name_table`: review table of the test colours.
- Reference data: edit `tools/references_source.txt`, run `python3 tools/gen_references.py`, then `npm run lint:data`.

## Storage and export (Phase 7)

- Data lives in `app.db` (SQLite) in the app data directory (macOS: `~/Library/Application Support/com.colorbuddy.app/`).
- Schema changes: append a script to `MIGRATIONS` in `src-tauri/src/storage.rs`; never edit an existing one.
- Export samples: `UPDATE_EXPORT_FIXTURES=1 cargo test -p color-buddy export` rewrites `src-tauri/tests/fixtures/exports/`; `src/test/exports.test.ts` validates them.

## Conventions

- The app UI must never rely on color alone; pair color with text/icons. WCAG AA text contrast.
- Offline only, no telemetry.
- CI (`.github/workflows/ci.yml`) runs lint + tests on macOS and Windows on every push.
