# tdo — repo guide

## Structure

Rust workspace (`resolver = "3"`, edition 2024), three crates:

| Crate | Type | Purpose |
|-------|------|---------|
| `crates/engine` | lib | SQLite-backed data store (rusqlite bundled, chrono). Schema migration on open, CRUD for projects & tasks. Defines `Engine`, `StoreError`, models. |
| `crates/tui` | lib | Placeholder — no real UI code yet. |
| `crates/tdo` | bin | Binary entrypoint — currently a stub. Depends on `engine`. |

The workspace root `Cargo.toml` lists all members; `Cargo.lock` is committed.

## Commands

```sh
cargo build              # whole workspace
cargo test               # all crates (only tui has a stub test currently)
cargo test -p <crate>    # single crate
cargo test <name>        # filter by test name
cargo fmt                # no config — uses rustfmt defaults
cargo clippy             # no config — uses clippy defaults
```

No CI, pre-commit hooks, or codegen scripts exist yet.

## Key details

- **SQLite schema** is created via `PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL;` then `CREATE TABLE IF NOT EXISTS ...` on every `Engine::open()`. There are no standalone migration files.
- `rusqlite` uses the `bundled` feature — no system libsqlite3 needed.
- `chrono` uses default feature set (no serde). `DateTime<Utc>` throughout.
- `tdo` binary currently does nothing — real CLI entrypoint and TUI crate are unwritten.
- `StoreError` is the domain error type across `engine`; callers match it for `NameTaken`, `TaskNameTaken`, `NotFound`, `Db`.
- Edition 2024 requires **Rust 1.85+**.
- No `rust-toolchain.toml` pins a version yet.
