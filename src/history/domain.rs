use serde::{Deserialize, Serialize};

use crate::version::Version;

#[derive(Debug, thiserror::Error)]
pub enum HistoryError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("could not resolve config directory")]
    ConfigDir,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub version: Version,
    pub summary: String,
    pub checked_at: jiff::Timestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct History {
    pub last_seen_version: Option<Version>,
    pub last_check: Option<jiff::Timestamp>,
    pub entries: Vec<HistoryEntry>,
}

pub trait HistoryRepository {
    fn load(&self) -> Result<History, HistoryError>;
    fn save(&self, state: &History) -> Result<History, HistoryError>;
}
