use std::path::PathBuf;

use super::domain::{History, HistoryError, HistoryRepository};

pub struct JsonHistoryRepository {
    path: PathBuf,
}

impl JsonHistoryRepository {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    /// Create a store at the default location (next to the binary).
    pub fn default_location() -> Result<Self, HistoryError> {
        let exe = std::env::current_exe().map_err(HistoryError::Io)?;
        let dir = exe.parent().ok_or(HistoryError::ConfigDir)?;
        Ok(Self::new(dir.join("claudesynth-history.json")))
    }
}

impl HistoryRepository for JsonHistoryRepository {
    fn load(&self) -> Result<History, HistoryError> {
        match std::fs::read_to_string(&self.path) {
            Ok(contents) => {
                let state: History = serde_json::from_str(&contents)?;
                Ok(state)
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(History::default()),
            Err(e) => Err(HistoryError::Io(e)),
        }
    }

    fn save(&self, state: &History) -> Result<History, HistoryError> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(state)?;
        std::fs::write(&self.path, json)?;
        Ok(state.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::history::HistoryEntry;

    fn temp_store() -> (tempfile::TempDir, JsonHistoryRepository) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("history.json");
        (dir, JsonHistoryRepository::new(path))
    }

    #[test]
    fn load_missing_file_returns_default() {
        let (_dir, store) = temp_store();
        let state = store.load().unwrap();
        assert!(state.last_seen_version.is_none());
        assert!(state.last_check.is_none());
        assert!(state.entries.is_empty());
    }

    #[test]
    fn save_then_load_roundtrips() {
        let (_dir, store) = temp_store();
        let now = jiff::Timestamp::now();
        let state = History {
            last_seen_version: Some("2.1.78".parse().unwrap()),
            last_check: Some(now),
            entries: vec![HistoryEntry {
                version: "2.1.78".parse().unwrap(),
                summary: "- New feature".to_string(),
                checked_at: now,
            }],
        };
        store.save(&state).unwrap();
        let loaded = store.load().unwrap();
        assert_eq!(loaded.last_seen_version.unwrap().to_string(), "2.1.78");
        assert!(loaded.last_check.is_some());
        assert_eq!(loaded.entries.len(), 1);
        assert_eq!(loaded.entries[0].version.to_string(), "2.1.78");
        assert_eq!(loaded.entries[0].summary, "- New feature");
    }

    #[test]
    fn load_corrupted_file_returns_error() {
        let (_dir, store) = temp_store();
        std::fs::write(&store.path, "not valid json{{{").unwrap();
        let result = store.load();
        assert!(result.is_err());
        assert!(
            matches!(result.unwrap_err(), HistoryError::Json(_)),
            "expected Json error variant"
        );
    }

    #[test]
    fn overwrite_existing_state() {
        let (_dir, store) = temp_store();
        let state1 = History {
            last_seen_version: Some("1.0.0".parse().unwrap()),
            last_check: None,
            entries: vec![],
        };
        store.save(&state1).unwrap();

        let now = jiff::Timestamp::now();
        let state2 = History {
            last_seen_version: Some("2.0.0".parse().unwrap()),
            last_check: Some(now),
            entries: vec![
                HistoryEntry {
                    version: "1.0.0".parse().unwrap(),
                    summary: "old".to_string(),
                    checked_at: now,
                },
                HistoryEntry {
                    version: "2.0.0".parse().unwrap(),
                    summary: "new".to_string(),
                    checked_at: now,
                },
            ],
        };
        store.save(&state2).unwrap();

        let loaded = store.load().unwrap();
        assert_eq!(loaded.last_seen_version.unwrap().to_string(), "2.0.0");
        assert_eq!(loaded.entries.len(), 2);
    }
}
