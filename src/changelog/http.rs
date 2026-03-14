use super::domain::{ChangelogError, ChangelogProvider, VersionEntry};
use super::parser::{new_versions_since, parse_changelog};
use crate::version::Version;

const CHANGELOG_URL: &str =
    "https://raw.githubusercontent.com/anthropics/claude-code/main/CHANGELOG.md";

pub struct HttpChangelog;

impl HttpChangelog {
    fn fetch_raw(&self) -> Result<String, ChangelogError> {
        let body = ureq::get(CHANGELOG_URL)
            .call()?
            .body_mut()
            .read_to_string()?;
        Ok(body)
    }
}

impl ChangelogProvider for HttpChangelog {
    fn fetch_newer_than(
        &self,
        last_seen: Option<&Version>,
    ) -> Result<Vec<VersionEntry>, ChangelogError> {
        let markdown = self.fetch_raw()?;
        let entries = parse_changelog(&markdown)?;
        let new_entries = new_versions_since(&entries, last_seen);
        Ok(new_entries)
    }
}
