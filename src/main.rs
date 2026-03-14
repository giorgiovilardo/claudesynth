mod changelog;
mod formatter;
mod history;
mod publisher;
mod summarizer;
mod version;

use clap::Parser;

use crate::changelog::{ChangelogProvider, HttpChangelog, VersionEntry};
use crate::formatter::{MarkdownSummaryFormatter, SummaryFormatter};
use crate::history::{HistoryRepository, JsonHistoryRepository};
use crate::publisher::{MessagePublisher, StdoutMessagePublisher};
use crate::summarizer::{ChangelogSummarizer, ClaudeSummarizer};

fn main() {
    if let Err(err) = run() {
        print_error(&err);
        std::process::exit(1);
    }
}

fn run() -> Result<(), AppError> {
    Cli::parse();

    let repo = JsonHistoryRepository::default_location().map_err(AppError::HistoryLocation)?;
    let history = repo.load().map_err(AppError::HistoryLoad)?;

    let new_entries = HttpChangelog.fetch_newer_than(history.last_seen_version.as_ref())?;
    if new_entries.is_empty() {
        println!("No new versions since last check.");
        return Ok(());
    }
    println!("Found {} new version(s).", new_entries.len());

    println!("Summarizing...");
    let summaries = ClaudeSummarizer::new()?.summarize(&new_entries)?;

    let items: Vec<_> = new_entries.into_iter().zip(summaries).collect();
    let message = MarkdownSummaryFormatter.format(&items)?;
    StdoutMessagePublisher.publish(&message)?;

    let new_history = updated_history(history, &items);
    repo.save(&new_history).map_err(AppError::HistorySave)?;

    Ok(())
}

#[derive(Parser, Debug)]
#[command(
    name = "claudesynth",
    version,
    about = "Claude Code changelog summary generator"
)]
struct Cli {}

#[derive(Debug, thiserror::Error)]
enum AppError {
    #[error("Failed to resolve history file location")]
    HistoryLocation(#[source] history::HistoryError),
    #[error("Failed to load history: {0}")]
    HistoryLoad(#[source] history::HistoryError),
    #[error("Failed to save history: {0}")]
    HistorySave(#[source] history::HistoryError),
    #[error("Failed to fetch changelog: {0}")]
    Changelog(#[from] changelog::ChangelogError),
    #[error("Failed to format message: {0}")]
    Format(#[from] formatter::FormatError),
    #[error("Failed to publish message: {0}")]
    Publish(#[from] publisher::PublishError),
    #[error("{0}")]
    Summarize(#[from] summarizer::SummarizeError),
}

fn updated_history(
    previous: history::History,
    items: &[(VersionEntry, summarizer::Summary)],
) -> history::History {
    let now = jiff::Timestamp::now();
    let mut entries = previous.entries;
    for (entry, summary) in items {
        entries.push(history::HistoryEntry {
            version: entry.version.clone(),
            summary: summary.text.clone(),
            checked_at: now,
        });
    }
    history::History {
        last_seen_version: Some(items[0].0.version.clone()),
        last_check: Some(now),
        entries,
    }
}

fn print_error(err: &AppError) {
    match err {
        AppError::Summarize(summarizer::SummarizeError::NotFound(_)) => {
            eprintln!(
                "Error: claude CLI not found. Install it from https://docs.anthropic.com/en/docs/claude-code"
            );
        }
        AppError::Summarize(summarizer::SummarizeError::EmptyOutput) => {
            eprintln!("Error: claude returned empty output. Try running again.");
        }
        AppError::Changelog(changelog::ChangelogError::Http(_)) => {
            eprintln!("Error: could not reach GitHub. Check your network connection.");
        }
        AppError::HistoryLoad(history::HistoryError::Json(_)) => {
            eprintln!(
                "Error: history file is corrupted. Delete claudesynth-history.json and retry."
            );
        }
        _ => eprintln!("Error: {err}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::summarizer::Summary;
    use crate::version::Version;
    use clap::CommandFactory;

    fn entry(version: &str, content: &str) -> VersionEntry {
        VersionEntry {
            version: version.parse::<Version>().unwrap(),
            content: content.to_string(),
        }
    }

    fn summary(text: &str) -> Summary {
        Summary {
            text: text.to_string(),
        }
    }

    #[test]
    fn cli_has_version_flag() {
        let cmd = Cli::command();
        assert_eq!(cmd.get_version(), Some(env!("CARGO_PKG_VERSION")));
    }

    #[test]
    fn updated_history_sets_last_seen_to_first_entry() {
        let previous = history::History::default();
        let items = vec![
            (entry("2.1.78", "newest"), summary("summary 78")),
            (entry("2.1.77", "older"), summary("summary 77")),
        ];

        let result = updated_history(previous, &items);

        assert_eq!(result.last_seen_version.unwrap().to_string(), "2.1.78");
    }

    #[test]
    fn updated_history_appends_one_entry_per_version() {
        let previous = history::History::default();
        let items = vec![
            (entry("2.1.78", "a"), summary("s78")),
            (entry("2.1.77", "b"), summary("s77")),
        ];

        let result = updated_history(previous, &items);

        assert_eq!(result.entries.len(), 2);
        assert_eq!(result.entries[0].version.to_string(), "2.1.78");
        assert_eq!(result.entries[1].version.to_string(), "2.1.77");
    }

    #[test]
    fn updated_history_preserves_previous_entries() {
        let previous = history::History {
            last_seen_version: Some("2.1.76".parse().unwrap()),
            last_check: Some(jiff::Timestamp::now()),
            entries: vec![history::HistoryEntry {
                version: "2.1.76".parse().unwrap(),
                summary: "old summary".to_string(),
                checked_at: jiff::Timestamp::now(),
            }],
        };
        let items = vec![(entry("2.1.77", "new"), summary("new summary"))];

        let result = updated_history(previous, &items);

        assert_eq!(result.entries.len(), 2);
        assert_eq!(result.entries[0].summary, "old summary");
        assert_eq!(result.entries[1].summary, "new summary");
    }

    #[test]
    fn updated_history_sets_last_check() {
        let previous = history::History::default();
        let items = vec![(entry("1.0.0", "x"), summary("s"))];

        let result = updated_history(previous, &items);

        assert!(result.last_check.is_some());
    }

    #[test]
    fn updated_history_stores_per_version_summary() {
        let previous = history::History::default();
        let items = vec![
            (entry("2.0.0", "a"), summary("summary for 2.0.0")),
            (entry("1.9.0", "b"), summary("summary for 1.9.0")),
        ];

        let result = updated_history(previous, &items);

        assert_eq!(result.entries[0].summary, "summary for 2.0.0");
        assert_eq!(result.entries[1].summary, "summary for 1.9.0");
    }
}
