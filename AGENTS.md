### Always respond in Russian.
# Repository Guidelines

## Project Structure & Module Organization
- Workspace root: `Cargo.toml` defines members:
  - `bar` - main GTK4 status bar application for Wayland
  - `modules/hyprland_workspaces` - module for working with Hyprland workspaces
  - `modules/time` - time utilities
  - `modules/lang` - keyboard layout/locale module
  - `modules/tray` - system tray module
  - `modules/helpers` - helper utilities
  - `modules/logger` - logging module
  - `modules/audio` - audio module (PulseAudio backend)
- `bar/src`: 
  - `main.rs` - entry point, GTK4 application initialization
  - `app.rs` - main application logic (BarApp)
  - `config.rs` - configuration
  - `services/` - services (hyprland.rs for Hyprland API integration)
  - `ui/` - UI components:
    - `components/` - clock.rs, lang.rs, tray.rs, workspaces.rs
    - `styles.rs` - styles
    - `window.rs` - application window
  - `resources/styles.css` - CSS styles
- `modules/audio/src`:
  - `lib.rs` - public module API
  - `main.rs` - entry point for standalone application
  - `backend/pulse/` - PulseAudio backend (client.rs, device_info.rs, listen_pulse_backend.rs, output_info.rs, start_listening.rs)
- Build artifacts land in `target/`; keep checked-in sources under `bar/` and `modules/`.

## Build, Test, and Development Commands
- Build all crates: `cargo build` (adds binaries/libraries to `target/`).
- Build only bar: `cargo build -p bar`.
- Run the bar (Wayland session required): `cargo run -p bar`.
- Run tests (workspace): `cargo test` (currently only `modules/time` has a unit test).

## Documentation

Module documentation is available in the `docs/` directory:
- [README](docs/README.md) - Overview and quick start
- [bar](docs/bar.md) - Main status bar application
- [hyprland_workspaces](docs/hyprland_workspaces.md) - Hyprland workspaces module
- [time](docs/time.md) - Time utilities
- [lang](docs/lang.md) - Keyboard layout module
- [tray](docs/tray.md) - System tray module
- [helpers](docs/helpers.md) - Helper utilities
- [logger](docs/logger.md) - Logging module
- [audio](docs/audio.md) - Audio module (PulseAudio backend)

**Documentation Language:**
- All documentation must be written in English.
- This includes markdown files in `docs/`, README files, and any other documentation files.
- Code examples and comments in documentation should also be in English.

## Coding Style & Naming Conventions
- Language: Rust 2024 edition; use `rustfmt` defaults (4-space indent, trailing commas where applicable).
- Keep modules small and focused; prefer `mod`-scoped helpers over sprawling files.
- Struct/enum names in `PascalCase`, functions in `snake_case`, constants in `SCREAMING_SNAKE_CASE`.
- Add brief comments only where intent is non-obvious (Wayland setup, Hyprland API assumptions).

**Code Comments Language:**
- All code comments must be written in English.
- This includes inline comments, doc comments (`///`), module-level comments, and any explanatory text in the code.
- Function and struct documentation should be in English.
- Error messages and log messages can be in English (preferred) or the user's locale, but code comments must be English.

## Testing Guidelines
- Use `cargo test` for unit/integration coverage; place unit tests in the same file under `#[cfg(test)]`.
- Name tests after the behavior under check (e.g., `parses_desktop_file_icon`, `maps_clients_to_workspace`).
- If adding Hyprland/Wayland integration, guard tests behind feature flags or mock external calls to keep CI headless-friendly.

## Commit & Pull Request Guidelines
- Commits: keep messages imperative and focused on a single change (e.g., `Add icon lookup for Hyprland clients`).
- Include context in PR descriptions: what changed, why, and how to verify (`cargo test`, `cargo run -p bar`).
- Attach screenshots/gifs only when altering visuals of the bar; otherwise list command outputs for verification.
- Reference related issues/tickets when available; note any environment needs (Wayland/Hyprland version expectations).

## Landing the Plane (Session Completion)

**When ending a work session**, you MUST complete ALL steps below. Work is NOT complete until `git push` succeeds.

**MANDATORY WORKFLOW:**

1. **File issues for remaining work** - Create issues for anything that needs follow-up
2. **Run quality gates** (if code changed) - Tests, linters, builds
3. **Update issue status** - Close finished work, update in-progress items
4. **PUSH TO REMOTE** - This is MANDATORY:
   ```bash
   git pull --rebase
   bd sync
   git push
   git status  # MUST show "up to date with origin"
   ```
5. **Clean up** - Clear stashes, prune remote branches
6. **Verify** - All changes committed AND pushed
7. **Hand off** - Provide context for next session

**CRITICAL RULES:**
- Work is NOT complete until `git push` succeeds
- NEVER stop before pushing - that leaves work stranded locally
- NEVER say "ready to push when you are" - YOU must push
- If push fails, resolve and retry until it succeeds

## Use 'bd' for task tracking
Use 'bd' for task tracking

<!-- bv-agent-instructions-v1 -->

---

## Beads Workflow Integration

This project uses [beads_viewer](https://github.com/Dicklesworthstone/beads_viewer) for issue tracking. Issues are stored in `.beads/` and tracked in git.

### Essential Commands

```bash
# View issues (launches TUI - avoid in automated sessions)
bv

# CLI commands for agents (use these instead)
bd ready              # Show issues ready to work (no blockers)
bd list --status=open # All open issues
bd show <id>          # Full issue details with dependencies
bd create --title="..." --type=task --priority=2
bd update <id> --status=in_progress
bd close <id> --reason="Completed"
bd close <id1> <id2>  # Close multiple issues at once
bd sync               # Commit and push changes
```

### Workflow Pattern

1. **Start**: Run `bd ready` to find actionable work
2. **Claim**: Use `bd update <id> --status=in_progress`
3. **Work**: Implement the task
4. **Complete**: Use `bd close <id>`
5. **Sync**: Always run `bd sync` at session end

### Key Concepts

- **Dependencies**: Issues can block other issues. `bd ready` shows only unblocked work.
- **Priority**: P0=critical, P1=high, P2=medium, P3=low, P4=backlog (use numbers, not words)
- **Types**: task, bug, feature, epic, question, docs
- **Blocking**: `bd dep add <issue> <depends-on>` to add dependencies

### Session Protocol

**Before ending any session, run this checklist:**

```bash
git status              # Check what changed
git add <files>         # Stage code changes
bd sync                 # Commit beads changes
git commit -m "..."     # Commit code
bd sync                 # Commit any new beads changes
git push                # Push to remote
```

### Best Practices

- Check `bd ready` at session start to find available work
- Update status as you work (in_progress → closed)
- Create new issues with `bd create` when you discover tasks
- Use descriptive titles and set appropriate priority/type
- Always `bd sync` before ending session

<!-- end-bv-agent-instructions -->
