### Всегда отвечай на русском языке.
# Repository Guidelines

## Project Structure & Module Organization
- Workspace root: `Cargo.toml` defines members `bar` (Wayland status bar) and `modules/time` (utility crate placeholder).
- `bar/src`: main application logic (`main.rs` initializes Wayland layer-shell bar), Wayland dispatch/state (`app_data.rs`), Hyprland client/workspace mapping (`hyprland_api.rs`), desktop entry parsing (`desktop_file.rs`), and icon discovery (`icon_fetcher.rs`).
- `modules/time/src`: currently minimal helper crate with a sample test; extend here for reusable time utilities.
- Build artifacts land in `target/`; keep checked-in sources under `bar/` and `modules/`.

## Build, Test, and Development Commands
- Build all crates: `cargo build` (adds binaries/libraries to `target/`).
- Build only bar: `cargo build -p bar`.
- Run the bar (Wayland session required): `cargo run -p bar`.
- Run tests (workspace): `cargo test` (currently only `modules/time` has a unit test).

## Coding Style & Naming Conventions
- Language: Rust 2024 edition; use `rustfmt` defaults (4-space indent, trailing commas where applicable).
- Keep modules small and focused; prefer `mod`-scoped helpers over sprawling files.
- Struct/enum names in `PascalCase`, functions in `snake_case`, constants in `SCREAMING_SNAKE_CASE`.
- Add brief comments only where intent is non-obvious (Wayland setup, Hyprland API assumptions).

## Testing Guidelines
- Use `cargo test` for unit/integration coverage; place unit tests in the same file under `#[cfg(test)]`.
- Name tests after the behavior under check (e.g., `parses_desktop_file_icon`, `maps_clients_to_workspace`).
- If adding Hyprland/Wayland integration, guard tests behind feature flags or mock external calls to keep CI headless-friendly.

## Commit & Pull Request Guidelines
- Commits: keep messages imperative and focused on a single change (e.g., `Add icon lookup for Hyprland clients`).
- Include context in PR descriptions: what changed, why, and how to verify (`cargo test`, `cargo run -p bar`).
- Attach screenshots/gifs only when altering visuals of the bar; otherwise list command outputs for verification.
- Reference related issues/tickets when available; note any environment needs (Wayland/Hyprland version expectations).
