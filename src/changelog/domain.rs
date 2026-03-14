use crate::version::Version;

#[derive(Debug, thiserror::Error)]
pub enum ChangelogError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] ureq::Error),
    #[error("failed to read response body: {0}")]
    Body(#[from] std::io::Error),
    #[error("no versions found in changelog")]
    NoVersions,
}

#[derive(Debug, Clone)]
pub struct VersionEntry {
    pub version: Version,
    pub content: String,
}

pub trait ChangelogProvider {
    fn fetch_newer_than(
        &self,
        last_seen: Option<&Version>,
    ) -> Result<Vec<VersionEntry>, ChangelogError>;
}
