# Repository Guidelines

## Project structure

Picaso is an image workbench with a Tauri desktop app and a standalone CLI. Put behavior both clients need in `core/`, which owns image jobs, providers, and the SQLite library. Edit `core/models.json` for bundled model defaults.

- `src/features/` contains the React workspace and onboarding screens. Shared controls live in `src/components/`, and `src/lib/desktop.ts` wraps calls to Tauri.
- `src-tauri/` contains native commands, credential handling, platform configuration, and app icons in `src-tauri/icons/`.
- `cli/src/` handles command parsing and output. Process-level command tests live in `cli/tests/commands.rs`.

## Build and run

Run these commands from the repository root. See `README.md` for platform prerequisites.

- `npm ci` installs the locked frontend dependencies.
- `npm run tauri -- dev` launches the desktop app. Append `--features fixture-provider` to exercise image workflows offline with a separate library.
- `cargo run -p picaso-cli -- models` runs the CLI and lists models offline.
- `npm run build` checks TypeScript and builds the frontend. Use `npm run tauri -- build` for the Windows installer or `npm run build:mac` for the Apple Silicon app and DMG.

## Coding style

Use strict TypeScript with two-space indentation, PascalCase React components, and camelCase functions. Use four-space indentation in Rust, snake_case functions and modules, and PascalCase types. Put each conditional branch and its body on separate lines. Write new UI text in English.

## Testing

- `npm test` runs Vitest. Place React Testing Library tests beside their components as `Component.test.tsx`. Use jsdom and mock native calls through the desktop wrapper.
- `cargo test --workspace` tests the core and CLI. Place Rust unit tests in a `tests` module and name cases after the behavior they verify.
- `cargo test -p picaso-cli --features fixture-provider` exercises offline CLI image workflows.
- `cargo test --manifest-path src-tauri/Cargo.toml` tests the desktop crate separately from the workspace.

Add a regression test for bug fixes when feasible. The repository defines no numeric coverage threshold.

## Commits and pull requests

Use a short imperative subject, following examples such as “Fix API key setup and shortcut hints on Windows.” In a pull request, describe the behavior change and list the checks you ran. Link related issues when applicable and include a screenshot for visible UI changes.
