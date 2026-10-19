# Color-Buddy — working notes for Claude Code

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

## Conventions

- The app UI must never rely on color alone; pair color with text/icons. WCAG AA text contrast.
- Offline only, no telemetry.
- CI (`.github/workflows/ci.yml`) runs lint + tests on macOS and Windows on every push.
