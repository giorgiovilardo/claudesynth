use crate::AppError;
use crate::history::HistoryRepository;
use crate::version::Version;

pub fn show(version: &Version, history_repo: &impl HistoryRepository) -> Result<(), AppError> {
    let history = history_repo.load().map_err(AppError::HistoryLoad)?;

    let entry = history.entries.iter().find(|e| &e.version == version);

    match entry {
        Some(entry) => {
            println!("{}", entry.summary);
            Ok(())
        }
        None => Err(AppError::VersionNotFound(version.clone())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::history::{History, HistoryEntry, HistoryError, HistoryRepository};

    struct MockHistoryRepo {
        history: History,
    }

    impl HistoryRepository for MockHistoryRepo {
        fn load(&self) -> Result<History, HistoryError> {
            Ok(self.history.clone())
        }

        fn save(&self, _state: &History) -> Result<History, HistoryError> {
            Ok(self.history.clone())
        }
    }

    fn repo_with_entries(entries: Vec<(&str, &str)>) -> MockHistoryRepo {
        let history_entries: Vec<HistoryEntry> = entries
            .into_iter()
            .map(|(v, s)| HistoryEntry {
                version: v.parse().unwrap(),
                summary: s.to_string(),
                checked_at: jiff::Timestamp::now(),
            })
            .collect();

        let last_seen = history_entries.first().map(|e| e.version.clone());

        MockHistoryRepo {
            history: History {
                last_seen_version: last_seen,
                last_check: Some(jiff::Timestamp::now()),
                entries: history_entries,
            },
        }
    }

    #[test]
    fn show_prints_summary_for_known_version() {
        let repo = repo_with_entries(vec![
            ("2.1.78", "Great new features in this release."),
            ("2.1.77", "Bug fixes and improvements."),
        ]);
        let version: Version = "2.1.78".parse().unwrap();

        let result = show(&version, &repo);

        assert!(result.is_ok());
    }

    #[test]
    fn show_returns_error_for_unknown_version() {
        let repo = repo_with_entries(vec![("2.1.78", "summary")]);
        let version: Version = "9.9.9".parse().unwrap();

        let result = show(&version, &repo);

        assert!(result.is_err());
    }

    #[test]
    fn show_returns_error_for_empty_history() {
        let repo = repo_with_entries(vec![]);
        let version: Version = "1.0.0".parse().unwrap();

        let result = show(&version, &repo);

        assert!(result.is_err());
    }
}
