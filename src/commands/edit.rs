use crate::AppError;
use crate::history::{HistoryEntry, HistoryRepository};
use crate::version::Version;

use super::editor::{EditOutcome, Editor};

pub fn edit(
    version: &Version,
    history_repo: &impl HistoryRepository,
    editor: &impl Editor,
) -> Result<(), AppError> {
    let mut history = history_repo.load().map_err(AppError::HistoryLoad)?;

    let current_summary = history
        .entries
        .iter()
        .find(|e| &e.version == version)
        .map(|e| e.summary.as_str())
        .unwrap_or("");

    let outcome = editor.edit(current_summary)?;

    match outcome {
        EditOutcome::Unchanged => {
            println!("No changes made.");
        }
        EditOutcome::Changed(new_summary) => {
            if let Some(entry) = history.entries.iter_mut().find(|e| &e.version == version) {
                entry.summary = new_summary;
                entry.checked_at = jiff::Timestamp::now();
            } else {
                history.entries.push(HistoryEntry {
                    version: version.clone(),
                    summary: new_summary,
                    checked_at: jiff::Timestamp::now(),
                });
            }
            history_repo.save(&history).map_err(AppError::HistorySave)?;
            println!("Summary updated for version {version}.");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::history::{History, HistoryEntry, HistoryError, HistoryRepository};

    use std::cell::RefCell;

    use super::super::editor::{EditError, EditOutcome, Editor};

    struct MockEditor {
        outcome: EditOutcome,
    }

    impl Editor for MockEditor {
        fn edit(&self, _initial_content: &str) -> Result<EditOutcome, EditError> {
            match &self.outcome {
                EditOutcome::Changed(s) => Ok(EditOutcome::Changed(s.clone())),
                EditOutcome::Unchanged => Ok(EditOutcome::Unchanged),
            }
        }
    }

    struct MockHistoryRepo {
        history: RefCell<History>,
        save_called: RefCell<bool>,
    }

    impl MockHistoryRepo {
        fn new(history: History) -> Self {
            Self {
                history: RefCell::new(history),
                save_called: RefCell::new(false),
            }
        }

        fn was_save_called(&self) -> bool {
            *self.save_called.borrow()
        }
    }

    impl HistoryRepository for MockHistoryRepo {
        fn load(&self) -> Result<History, HistoryError> {
            Ok(self.history.borrow().clone())
        }

        fn save(&self, state: &History) -> Result<History, HistoryError> {
            *self.save_called.borrow_mut() = true;
            *self.history.borrow_mut() = state.clone();
            Ok(state.clone())
        }
    }

    fn history_with_entries(entries: Vec<(&str, &str)>) -> History {
        let history_entries: Vec<HistoryEntry> = entries
            .into_iter()
            .map(|(v, s)| HistoryEntry {
                version: v.parse().unwrap(),
                summary: s.to_string(),
                checked_at: jiff::Timestamp::now(),
            })
            .collect();

        let last_seen = history_entries.first().map(|e| e.version.clone());

        History {
            last_seen_version: last_seen,
            last_check: Some(jiff::Timestamp::now()),
            entries: history_entries,
        }
    }

    #[test]
    fn existing_version_changed_updates_entry() {
        let history = history_with_entries(vec![("2.1.78", "old summary")]);
        let repo = MockHistoryRepo::new(history);
        let editor = MockEditor {
            outcome: EditOutcome::Changed("new summary".to_string()),
        };
        let version: Version = "2.1.78".parse().unwrap();

        edit(&version, &repo, &editor).unwrap();

        assert!(repo.was_save_called());
        let saved = repo.history.borrow();
        assert_eq!(saved.entries[0].summary, "new summary");
    }

    #[test]
    fn existing_version_unchanged_does_not_save() {
        let history = history_with_entries(vec![("2.1.78", "summary")]);
        let repo = MockHistoryRepo::new(history);
        let editor = MockEditor {
            outcome: EditOutcome::Unchanged,
        };
        let version: Version = "2.1.78".parse().unwrap();

        edit(&version, &repo, &editor).unwrap();

        assert!(!repo.was_save_called());
    }

    #[test]
    fn new_version_changed_creates_entry() {
        let history = history_with_entries(vec![("2.1.78", "existing")]);
        let repo = MockHistoryRepo::new(history);
        let editor = MockEditor {
            outcome: EditOutcome::Changed("brand new".to_string()),
        };
        let version: Version = "9.9.9".parse().unwrap();

        edit(&version, &repo, &editor).unwrap();

        assert!(repo.was_save_called());
        let saved = repo.history.borrow();
        assert_eq!(saved.entries.len(), 2);
        let new_entry = saved
            .entries
            .iter()
            .find(|e| e.version.to_string() == "9.9.9")
            .unwrap();
        assert_eq!(new_entry.summary, "brand new");
    }

    #[test]
    fn new_version_unchanged_does_not_save() {
        let history = history_with_entries(vec![]);
        let repo = MockHistoryRepo::new(history);
        let editor = MockEditor {
            outcome: EditOutcome::Unchanged,
        };
        let version: Version = "9.9.9".parse().unwrap();

        edit(&version, &repo, &editor).unwrap();

        assert!(!repo.was_save_called());
    }
}
