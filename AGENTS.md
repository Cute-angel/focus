# Repository Guidelines

## Project Structure & Module Organization
This repository is a desktop app built with Vue 3 + TypeScript (frontend) and Tauri + Rust (backend).

- `src/`: Vue app code (`pages/`, `components/`, `composables/`, `api/`).
- `public/`: static web assets used by Vite.
- `src-tauri/src/`: Rust application core (`api/`, `core/`, `plugins/`, `commands.rs`).
- `src-tauri/icons/` and `src/assets/`: app icons and visual assets.
- `dist/` and `target/`: build outputs; do not edit manually.

## Build, Test, and Development Commands
Use `pnpm` (lockfile is `pnpm-lock.yaml`).

- `pnpm dev`: run the Vite frontend in development mode.
- `pnpm build`: type-check (`vue-tsc`) and build production frontend assets.
- `pnpm preview`: preview the production frontend build locally.
- `pnpm tauri dev`: run the full desktop app (frontend + Tauri backend).
- `pnpm tauri build`: build desktop binaries.
- `cd src-tauri && cargo test`: run Rust unit/integration tests.

## Coding Style & Naming Conventions
- TypeScript uses strict settings (`tsconfig.json`): keep code type-safe and avoid unused locals/params.
- Vue SFCs: component files in `PascalCase` (e.g., `QueryPage.vue`, `ResultItem.vue`).
- Keep existing per-file style conventions (quote style and indentation) when modifying legacy files.
- Rust: follow idiomatic module organization and run `cargo fmt` before submitting backend changes.
- Prefer clear, small modules: frontend logic in `composables/` or `api/`; backend logic in `core/` or `plugins/`.

## Testing Guidelines
There is no dedicated JS test runner configured yet. For now:

- Add Rust tests near the code (`mod tests`) or under `src-tauri/tests/`.
- Name tests by behavior (example: `returns_results_for_prefix_query`).
- Validate UI changes manually with `pnpm tauri dev`.
- If you add JS tests, document the command in `package.json` and update this guide.

## Commit & Pull Request Guidelines
Commits follow Conventional Commits with `cz-git` and emoji style (see `commitlint.config.js`), e.g.:

- `feat(config): :sparkles: add config helper`
- `fix(launcher): :bug: handle invalid app path`

PRs should include:

- A concise description of user-visible and technical changes.
- Linked issue(s) when applicable.
- Screenshots or short recordings for UI updates.
- Notes on local verification steps (commands run, platforms tested).
