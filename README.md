# Color-Buddy

A desktop app that helps colorblind designers pick, name and combine colors with confidence,
without relying on their own color perception.

Built with Tauri 2, React 18 + TypeScript and a Rust color core. See [`SPEC.md`](SPEC.md) for
the full specification and roadmap.

## Prerequisites

- [Node.js](https://nodejs.org/) 22+ and npm
- [Rust](https://rustup.rs/) stable (with `rustfmt` and `clippy`)
- Platform prerequisites for Tauri: <https://tauri.app/start/prerequisites/>
  - **macOS 13+:** Xcode Command Line Tools (`xcode-select --install`)
  - **Windows 10/11:** Microsoft C++ Build Tools and WebView2 (preinstalled on Windows 11)

## Setup

```sh
npm install
npm run tauri dev   # opens the app window
```

## Checks

```sh
npm run lint        # ESLint + Prettier
npm run typecheck   # tsc
npm test            # Vitest
npm run build       # builds the frontend into dist/ (needed before the Rust checks)
npm run lint:rust   # cargo fmt --check + cargo clippy -D warnings
npm run test:rust   # cargo test --workspace
```

`npm run check` runs all of the above except `build`.

## Project layout

```
src/                 React UI
src-tauri/           Tauri backend
crates/color-core/   Pure Rust color logic (no Tauri dependency)
```
