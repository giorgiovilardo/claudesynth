# claudesynth

Rust CLI that fetches the [Claude Code changelog](https://github.com/anthropics/claude-code/blob/main/CHANGELOG.md), summarizes new versions via `claude -p`, and outputs a Slack-ready markdown digest.

## Usage

```
claudesynth run          # fetch, summarize, and publish new versions
claudesynth show 2.1.78  # print the stored summary for a specific version
claudesynth edit 2.1.78  # edit the stored summary in $EDITOR
```

Requires [Claude Code](https://docs.anthropic.com/en/docs/claude-code) (`claude` CLI) to be installed and available on PATH.

## How it works

The `run` command:

```
Load history
  → Fetch changelog from GitHub
  → Diff against last seen version (first run: latest 3)
  → Summarize new entries via `claude -p`
  → Format as markdown message
  → Publish (currently stdout)
  → Save updated history
```

The `show` command looks up a version in the saved history and prints its stored summary.

The `edit` command opens the stored summary for a version in `$EDITOR` (or `$VISUAL`). If the content changes on save, the history entry is updated. If the version doesn't exist yet, a new entry is created.

## Architecture

```
version.rs                   — semver Version type (FromStr, Display, Ord, Serde)
main.rs                      — Cli struct, AppError enum, command dispatcher

commands/                    — subcommand implementations (trait-based DI)
  commands.rs                — Command enum (clap Subcommand), re-exports
  run.rs                     — full pipeline (fetch → summarize → format → publish → save)
  show.rs                    — look up a version in history, print its summary
  edit.rs                    — open stored summary in $EDITOR, save changes
  editor.rs                  — Editor trait, EditOutcome, EnvEditor ($EDITOR/$VISUAL)

changelog/                   — fetch & parse the Claude Code changelog
  domain.rs                  — ChangelogError, ChangelogProvider trait, VersionEntry
  http.rs                    — HttpChangelog (ureq fetch from GitHub)
  parser.rs                  — pulldown-cmark parser, version diffing

summarizer/                  — LLM summarization of changelog entries
  domain.rs                  — Summary, ChangelogSummarizer trait, SummarizeError
  claude.rs                  — ClaudeSummarizer (shells out to `claude -p`)

formatter/                   — format summary into publishable message
  domain.rs                  — Message, SummaryFormatter trait, FormatError
  markdown.rs                — MarkdownSummaryFormatter (builds final message with header/footer)

publisher/                   — output abstraction
  domain.rs                  — PublishError, MessagePublisher trait
  stdout.rs                  — StdoutMessagePublisher (prints to stdout)

history/                     — persistent run history
  domain.rs                  — History, HistoryEntry, HistoryRepository trait, HistoryError
  json_file.rs               — JsonHistoryRepository (claudesynth-history.json)
```

Six trait-based abstractions (`ChangelogProvider`, `ChangelogSummarizer`, `SummaryFormatter`, `MessagePublisher`, `HistoryRepository`, `Editor`) allow swapping backends without changing the pipeline.

## State

Persisted as `claudesynth-history.json` next to the binary (resolved via `current_exe()`):

```json
{
  "last_seen_version": "2.1.78",
  "last_check": "2026-03-14T10:30:00Z",
  "entries": [
    {
      "version": "2.1.78",
      "summary": "- New feature X\n- Bug fix Y",
      "checked_at": "2026-03-14T10:30:00Z"
    }
  ]
}
```

## Prompt customization

On first run, a `prompt.txt` file is created next to the binary with a default summarization prompt. Edit this file to customize how `claude -p` summarizes changelog entries.

## Development

Requires Rust (edition 2024) and [just](https://github.com/casey/just).

```
just            # list available recipes
just qa         # fmt + clippy + test + check (must pass clean)
just test       # cargo test
just lint       # cargo clippy -- -D warnings
just fmt        # cargo fmt
just build      # cargo build
just release    # cargo build --release (stripped, LTO)
just run        # run the full pipeline (claudesynth run)
just clean      # cargo clean
```

## CI

GitHub Actions runs on pushes to `main` and on pull requests: format check, clippy, test, build.
