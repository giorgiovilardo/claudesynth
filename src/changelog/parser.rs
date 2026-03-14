use pulldown_cmark::{Event, HeadingLevel, Parser, Tag, TagEnd};

use super::domain::{ChangelogError, VersionEntry};
use crate::version::Version;

/// Parse a changelog markdown string into version entries (newest first).
pub(super) fn parse_changelog(markdown: &str) -> Result<Vec<VersionEntry>, ChangelogError> {
    let parser = Parser::new(markdown);

    // Collect byte offsets of h2 headings and their text
    let mut h2_positions: Vec<(usize, String)> = Vec::new();
    let mut in_h2 = false;
    let mut current_heading = String::new();
    let mut heading_start: usize = 0;

    for (event, range) in parser.into_offset_iter() {
        match event {
            Event::Start(Tag::Heading {
                level: HeadingLevel::H2,
                ..
            }) => {
                in_h2 = true;
                current_heading.clear();
                heading_start = range.start;
            }
            Event::End(TagEnd::Heading(HeadingLevel::H2)) => {
                in_h2 = false;
                h2_positions.push((heading_start, current_heading.clone()));
            }
            Event::Text(text) if in_h2 => {
                current_heading.push_str(&text);
            }
            Event::Code(code) if in_h2 => {
                current_heading.push_str(&code);
            }
            _ => {}
        }
    }

    // Extract content between consecutive h2s
    let mut entries = Vec::new();
    for (i, (start, heading_text)) in h2_positions.iter().enumerate() {
        let version_str = heading_text.trim();
        let version = match version_str.parse::<Version>() {
            Ok(v) => v,
            Err(_) => continue,
        };

        let content_start = markdown[*start..]
            .find('\n')
            .map(|pos| start + pos + 1)
            .unwrap_or(markdown.len());

        let content_end = if i + 1 < h2_positions.len() {
            h2_positions[i + 1].0
        } else {
            markdown.len()
        };

        let content = markdown[content_start..content_end].trim().to_string();
        entries.push(VersionEntry { version, content });
    }

    if entries.is_empty() {
        return Err(ChangelogError::NoVersions);
    }

    Ok(entries)
}

/// Return entries newer than `last_seen`.
/// - If `last_seen` is None: return latest 3
/// - If `last_seen` found: return all entries before it (newer)
/// - If `last_seen` not found: return up to 5 with a warning log
pub(super) fn new_versions_since(
    entries: &[VersionEntry],
    last_seen: Option<&Version>,
) -> Vec<VersionEntry> {
    let Some(last_seen) = last_seen else {
        return entries.iter().take(3).cloned().collect();
    };

    let mut result = Vec::new();
    for entry in entries {
        if entry.version == *last_seen {
            return result;
        }
        result.push(entry.clone());
    }

    eprintln!(
        "warning: last seen version {last_seen} not found in changelog, returning up to 5 entries"
    );
    result.truncate(5);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_CHANGELOG: &str = "\
# Changelog

## 2.1.78

- Added new feature X
- Fixed bug Y

## 2.1.77

- Improved performance
- Updated dependency Z

## 2.1.76

- Initial release notes
- Some other change

## 2.1.75

- Old stuff
";

    #[test]
    fn parse_extracts_versions() {
        let entries = parse_changelog(SAMPLE_CHANGELOG).unwrap();
        assert_eq!(entries.len(), 4);
        assert_eq!(entries[0].version.to_string(), "2.1.78");
        assert_eq!(entries[1].version.to_string(), "2.1.77");
        assert_eq!(entries[2].version.to_string(), "2.1.76");
        assert_eq!(entries[3].version.to_string(), "2.1.75");
    }

    #[test]
    fn parse_extracts_content() {
        let entries = parse_changelog(SAMPLE_CHANGELOG).unwrap();
        assert!(entries[0].content.contains("Added new feature X"));
        assert!(entries[0].content.contains("Fixed bug Y"));
    }

    #[test]
    fn parse_no_versions_returns_error() {
        let md = "# Just a title\n\nSome text.";
        assert!(parse_changelog(md).is_err());
    }

    #[test]
    fn parse_skips_non_version_headings() {
        let md = "\
## 2.1.78

- Feature

## Not A Version

- Stuff

## 2.1.77

- Fix
";
        let entries = parse_changelog(md).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].version.to_string(), "2.1.78");
        assert_eq!(entries[1].version.to_string(), "2.1.77");
    }

    #[test]
    fn parse_single_version() {
        let md = "## 1.0.0\n\n- Only version\n- Another change\n";
        let entries = parse_changelog(md).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].version.to_string(), "1.0.0");
        assert!(entries[0].content.contains("Only version"));
        assert!(entries[0].content.contains("Another change"));
    }

    #[test]
    fn new_versions_none_returns_latest_3() {
        let entries = parse_changelog(SAMPLE_CHANGELOG).unwrap();
        let new = new_versions_since(&entries, None);
        assert_eq!(new.len(), 3);
        assert_eq!(new[0].version.to_string(), "2.1.78");
        assert_eq!(new[2].version.to_string(), "2.1.76");
    }

    #[test]
    fn new_versions_since_known() {
        let entries = parse_changelog(SAMPLE_CHANGELOG).unwrap();
        let last_seen: Version = "2.1.76".parse().unwrap();
        let new = new_versions_since(&entries, Some(&last_seen));
        assert_eq!(new.len(), 2);
        assert_eq!(new[0].version.to_string(), "2.1.78");
        assert_eq!(new[1].version.to_string(), "2.1.77");
    }

    #[test]
    fn new_versions_since_latest_returns_empty() {
        let entries = parse_changelog(SAMPLE_CHANGELOG).unwrap();
        let last_seen: Version = "2.1.78".parse().unwrap();
        let new = new_versions_since(&entries, Some(&last_seen));
        assert!(new.is_empty());
    }

    #[test]
    fn new_versions_since_not_found_returns_up_to_5() {
        let entries = parse_changelog(SAMPLE_CHANGELOG).unwrap();
        let last_seen: Version = "1.0.0".parse().unwrap();
        let new = new_versions_since(&entries, Some(&last_seen));
        assert_eq!(new.len(), 4); // only 4 entries exist, cap is 5
    }
}
