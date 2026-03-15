# CLAUDE.md

## What is this

`claudesynth` is a Rust CLI that fetches the Claude Code CHANGELOG.md from GitHub, checks if already summarized, if not summarize it via LLM, and output a synthesis.

## Language & toolchain

- Rust, edition 2024, synchronous (no async runtime for now)
- VCS: `jj` (Jujutsu), **never** raw `git` for commits/diffs/logs
- Build runner: `just` (see justfile)

## Dependencies

| Crate | Purpose |
|---|---|
| `ureq` | Blocking HTTP client (fetch changelog from GitHub) |
| `clap` (derive) | CLI argument parsing |
| `jiff` | Human-readable timestamps (serde-compatible) |
| `pulldown-cmark` | Markdown parsing for changelog extraction |
| `serde` + `serde_json` | State serialization |
| `thiserror` | Error enums (per-module + `AppError` in main) |
| `tempfile` | Temp files for editor command + test temp dirs |

## Architecture

Check the @README.md file for guidance.

### Key patterns

- **Trait-based abstractions** for swappable backends: `ChangelogProvider`, `ChangelogSummarizer`, `SummaryFormatter`, `MessagePublisher`, `HistoryRepository`, `Editor`
- **`domain.rs` + implementation** pattern: each module separates trait definitions and error types (`domain.rs`) from concrete implementations
- Each module has its own `thiserror` error enum; `main.rs` uses a custom `AppError` enum (no anyhow)

### Commands

- **`run`** — full pipeline: fetch → diff → summarize → format → publish → save history
- **`show <version>`** — look up a version in history and print its stored summary
- **`edit <version>`** — open stored summary in `$EDITOR`/`$VISUAL`, save changes back to history

## Conventions

- **TDD**: red-green-refactor, baby steps, unit tests in every module (`#[cfg(test)] mod tests`)
- **Do not artificially suppress warning** — no escape hatches like `#[allow(unused_*)]`; fix the root cause. If in need of help, ask the user
- **No vanity fields** — only include struct fields that serve a concrete purpose
- Small commits, with `jj`
- `just qa` must pass clean before committing (fmt + clippy + test + check)
- No `mod.rs`: for modules, use the new style `modulename.rs` + `modulename/`
- If you need to use `grep`, use `rg` (ripgrep)
- If you need to use `find`, use `fd` (fd-find)
- Update README.md and CLAUDE.md, if needed, after every feature.

## Running

```
just qa          # full quality check (fmt + clippy + test + check)
just test        # unit tests only
just lint        # clippy only
just run         # full pipeline
```
