use crate::AppError;
use crate::changelog::{ChangelogProvider, VersionEntry};
use crate::formatter::SummaryFormatter;
use crate::history::{History, HistoryEntry, HistoryRepository};
use crate::publisher::MessagePublisher;
use crate::summarizer::{ChangelogSummarizer, Summary};

pub fn run(
    changelog: &impl ChangelogProvider,
    summarizer: &impl ChangelogSummarizer,
    formatter: &impl SummaryFormatter,
    publisher: &impl MessagePublisher,
    history_repo: &impl HistoryRepository,
) -> Result<(), AppError> {
    let history = history_repo.load().map_err(AppError::HistoryLoad)?;

    let new_entries = changelog.fetch_newer_than(history.last_seen_version.as_ref())?;
    if new_entries.is_empty() {
        println!("No new versions since last check.");
        return Ok(());
    }
    println!("Found {} new version(s).", new_entries.len());

    println!("Summarizing...");
    let summaries = summarizer.summarize(&new_entries)?;

    let items: Vec<_> = new_entries.into_iter().zip(summaries).collect();
    let message = formatter.format(&items)?;
    publisher.publish(&message)?;

    let new_history = updated_history(history, &items);
    history_repo
        .save(&new_history)
        .map_err(AppError::HistorySave)?;

    Ok(())
}

fn updated_history(previous: History, items: &[(VersionEntry, Summary)]) -> History {
    let now = jiff::Timestamp::now();
    let mut entries = previous.entries;
    for (entry, summary) in items {
        entries.push(HistoryEntry {
            version: entry.version.clone(),
            summary: summary.text.clone(),
            checked_at: now,
        });
    }
    History {
        last_seen_version: Some(items[0].0.version.clone()),
        last_check: Some(now),
        entries,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::{summary, version_entry as entry};

    #[test]
    fn updated_history_sets_last_seen_to_first_entry() {
        let previous = History::default();
        let items = vec![
            (entry("2.1.78", "newest"), summary("summary 78")),
            (entry("2.1.77", "older"), summary("summary 77")),
        ];

        let result = updated_history(previous, &items);

        assert_eq!(result.last_seen_version.unwrap().to_string(), "2.1.78");
    }

    #[test]
    fn updated_history_appends_one_entry_per_version() {
        let previous = History::default();
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
        let previous = History {
            last_seen_version: Some("2.1.76".parse().unwrap()),
            last_check: Some(jiff::Timestamp::now()),
            entries: vec![HistoryEntry {
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
        let previous = History::default();
        let items = vec![(entry("1.0.0", "x"), summary("s"))];

        let result = updated_history(previous, &items);

        assert!(result.last_check.is_some());
    }

    #[test]
    fn updated_history_stores_per_version_summary() {
        let previous = History::default();
        let items = vec![
            (entry("2.0.0", "a"), summary("summary for 2.0.0")),
            (entry("1.9.0", "b"), summary("summary for 1.9.0")),
        ];

        let result = updated_history(previous, &items);

        assert_eq!(result.entries[0].summary, "summary for 2.0.0");
        assert_eq!(result.entries[1].summary, "summary for 1.9.0");
    }
}
