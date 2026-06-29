# SaveSentry - Agent Context

**Stack**: Rust + egui + SQLite | **Platform**: Windows only | **Size**: ~75KB source

## Structure

```
src/
├── main.rs              # Entry: eframe::run_native → ui::App::new()
├── ui/                  # Presentation (immediate-mode egui)
│   ├── app.rs           # Orchestration only (~70 loc). Composes components in update()
│   ├── state.rs         # AppState: all fields pub. DB, templates, active_profile, watcher
│   ├── actions/         # Business logic
│   │   ├── monitoring.rs # start/stop monitoring, select_template, restore_backup
│   │   └── templates.rs  # CRUD for game templates
│   ├── components/      # Shared UI: tab_bar, messages
│   └── pages/           # 3 tabs: main, templates, settings
├── models/              # Domain: GameProfile, GameTemplate
├── db/                  # SQLite repository + refinery migrations
└── watcher/             # Background file watching + process monitoring
    ├── simple_watcher.rs # Thread-based notify watcher + mpsc channels
    ├── file_watcher.rs   # Backup logic: timeout, regex exclude, ZIP creation
    └── process_monitor.rs # sysinfo-based hybrid polling (1s/10s)
```

## Where to Look

| Task | Location |
|------|----------|
| Add new UI component | `src/ui/components/` or `src/ui/pages/<tab>/` |
| Change backup logic | `src/watcher/file_watcher.rs` |
| Add game template | `src/db/migrations/` (V{n}__desc.sql) |
| Change database schema | `src/db/migrations/` + `src/db/database.rs` |
| Fix UI state flow | `src/ui/state.rs` |
| Add business logic | `src/ui/actions/` |
| Change window behavior | `src/main.rs` |
| Release build config | `Cargo.toml` `[profile.release-windows]` |

## Code Map

| Symbol | Type | File | Role |
|--------|------|------|------|
| `App` | struct | `ui/app.rs` | eframe::App implementation |
| `AppState` | struct | `ui/state.rs` | Central mutable state |
| `GameProfile` | struct | `models/game_profile.rs` | Save configuration entity |
| `GameTemplate` | struct | `models/game_template.rs` | Pre-configured game preset |
| `Database` | struct | `db/database.rs` | SQLite repository |
| `FileWatcher` | struct | `watcher/file_watcher.rs` | Backup creation logic |
| `WatcherHandle` | struct | `watcher/simple_watcher.rs` | Background thread handle |
| `ProcessMonitor` | struct | `watcher/process_monitor.rs` | Process detection |
| `start_watching` | fn | `watcher/simple_watcher.rs` | Spawns file + process threads |
| `select_template` | fn | `ui/actions/monitoring.rs` | Sets active profile from template |
| `create_backup` | fn | `watcher/file_watcher.rs` | Creates timestamped ZIP |

## Conventions

- **Immediate-mode UI**: State and rendering unified. No Elm architecture.
- **app.rs**: Orchestration only. Never business logic.
- **state.rs**: All fields `pub`. Simple helpers only (< 5 lines).
- **actions.rs**: `impl AppState` methods. Validate, interact with DB, manage watchers.
- **components**: Pure functions `render_xxx(ui, state)`. No business logic.
- **Borrow checker**: Clone data BEFORE egui loops to avoid borrow issues.
- **Error handling**: `anyhow::Result` for fallible. User messages in `state.error_message`.
- **No unwrap/expect** in production paths (except initialization).
- **Strings**: User-facing text in Portuguese (pt-BR).
- **Timestamps**: `DD-MM-YYYY_HH-MM-SS` format.

## Anti-Patterns

- Do NOT add logic to `app.rs` — only component composition.
- Do NOT put DB/watcher logic in `state.rs`.
- Do NOT iterate `state.xxx` directly in egui loops — clone first.
- Do NOT modify existing migrations — create new V{n+1}__desc.sql.
- Do NOT delete official templates (`is_official=1`).
- No `println!` in release — use `#[cfg(debug_assertions)]`.

## Commands

```powershell
# Dev
cargo check
cargo build
cargo run

# Quality
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings
cargo test

# Release (size-optimized, target < 6MB)
cargo build --profile release-windows

# Makefile shortcuts
make validate    # fmt + clippy + check
make build-windows   # release build + size check
```

## Notes

- **Single watcher only**: `AppState.active_watcher: Option<WatcherHandle>`.
- **Auto-restore**: Last profile + config restored from DB on startup. Auto-starts watcher if `process_name` set.
- **Process-aware monitoring**: If `process_name` exists, file watching only runs while process is detected.
- **Backup cache**: 5s TTL on backup history listing.
- **Save info throttle**: 2s max refresh rate.
- **Portable**: DB at `<exe_dir>/savesentry.db`. No registry.
- **Windows subsystem**: `build.rs` sets `/SUBSYSTEM:WINDOWS` for release.


