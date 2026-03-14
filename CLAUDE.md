# CLAUDE.md

## What is this

claudesynth is a Rust CLI that fetches the Claude Code CHANGELOG.md from GitHub, diffs against last-seen versions, summarizes new entries via `claude -p`, and outputs Slack-ready markdown messages.

## Language & toolchain

- Rust, edition 2024, synchronous (no async runtime)
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
| `tempfile` (dev) | Temp dirs for storage tests |

## Architecture

```
version.rs                   — semver Version type (FromStr, Display, Ord, Serde)
main.rs                      — Cli struct, AppError enum, pipeline orchestration

changelog/                   — fetch & parse the Claude Code changelog
  domain.rs                  — ChangelogError, ChangelogProvider trait, VersionEntry
  http.rs                    — HttpChangelog (ureq fetch from GitHub)
  parser.rs                  — pulldown-cmark parser → Vec<VersionEntry>, version diffing

summarizer/                  — LLM summarization of changelog entries
  domain.rs                  — Summary, ChangelogSummarizer trait, SummarizeError
  claude.rs                  — ClaudeSummarizer (shells out to `claude -p` with prompt.txt)

formatter/                   — format summary into publishable message
  domain.rs                  — Message, SummaryFormatter trait, FormatError
  markdown.rs                — MarkdownSummaryFormatter (builds final message with header/footer)

publisher/                   — output abstraction
  domain.rs                  — PublishError, MessagePublisher trait
  stdout.rs                  — StdoutMessagePublisher (prints to stdout)

history/                     — persistent run history
  domain.rs                  — History, HistoryEntry, HistoryRepository trait, HistoryError
  json_file.rs               — JsonHistoryRepository (persists as claudesynth-history.json)
```

### Key patterns

- **Trait-based abstractions** for swappable backends: `ChangelogProvider`, `ChangelogSummarizer`, `SummaryFormatter`, `MessagePublisher`, `HistoryRepository`
- **`domain.rs` + implementation** pattern: each module separates trait definitions and error types (`domain.rs`) from concrete implementations
- Each module has its own `thiserror` error enum; `main.rs` uses a custom `AppError` enum (no anyhow)
- History file (`claudesynth-history.json`) and `prompt.txt` live next to the binary via `std::env::current_exe()`

### Pipeline flow

1. Load history from `HistoryRepository`
2. Fetch & diff via `ChangelogProvider::fetch_newer_than()` — returns only new `VersionEntry`s (on first run: latest 3)
3. Summarize via `ChangelogSummarizer` (currently shells out to `claude -p`)
4. Format into publishable message via `SummaryFormatter`
5. Publish via `MessagePublisher` (currently stdout)
6. Save updated history with per-version entries

## Conventions

- **TDD**: red-green-refactor, baby steps, unit tests in every module (`#[cfg(test)] mod tests`)
- **Never suppress warnings** — no `#[allow(unused_*)]`; fix the root cause (remove dead code, use the import, don't export until needed)
- **No vanity fields** — only include struct fields that serve a concrete purpose
- Commit atomically per module/feature
- `just qa` must pass clean before committing (fmt + clippy + test + check)
- Slack renders standard markdown — no Slack-specific mrkdwn formatting needed

## Who works here

Giorgio is an experienced developer who is new to Rust. He's comfortable with architecture concepts (traits, modules, error handling) but still building familiarity with Rust-specific libraries and idioms. He prefers clean abstractions and trait-based designs. Frame Rust explanations in general programming terms where possible.

## Running

```
just qa          # full quality check (fmt + clippy + test + check)
just test        # unit tests only
just lint        # clippy only
just run         # full pipeline
```

## CI

GitHub Actions (`.github/workflows/ci.yml`): fmt --check, clippy, test, build on ubuntu-latest with stable Rust.
