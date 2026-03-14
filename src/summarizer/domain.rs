use crate::changelog::VersionEntry;

#[derive(Debug, thiserror::Error)]
pub enum SummarizeError {
    #[error("claude binary not found: {0}")]
    NotFound(std::io::Error),
    #[error("claude process failed with status {status}: {stderr}")]
    ProcessFailed { status: i32, stderr: String },
    #[error("empty output from claude")]
    EmptyOutput,
    #[error("failed to read prompt file: {0}")]
    PromptFile(std::io::Error),
    #[error("invalid JSON in claude output: {0}")]
    InvalidJson(#[from] serde_json::Error),
}

#[derive(Debug, Clone)]
pub struct Summary {
    pub text: String,
}

pub trait ChangelogSummarizer {
    fn summarize(&self, entries: &[VersionEntry]) -> Result<Vec<Summary>, SummarizeError>;
}
