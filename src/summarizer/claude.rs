use std::path::PathBuf;
use std::process::Command;

use serde::Deserialize;

use crate::changelog::VersionEntry;
use crate::version::Version;

use super::domain::{ChangelogSummarizer, SummarizeError, Summary};

const DEFAULT_PROMPT: &str = "\
You are writing a casual patch commentary for a developer channel about new Claude Code releases.
The audience is developers who follow updates closely.

For each version, write a summary of the notable changes.
Group changes by category when it makes sense:
- Features / new capabilities
- Bug fixes
- Improvements / QoL changes

For each notable item, add a brief editorial comment — what it means in practice,
why it matters, or who benefits. Use a neutral third-person perspective, not first-person.
Example style:
- Fixed `/resume` showing the current session in the picker — big one for heavy /resume users, the menu should be much clearer now
- Improved clipboard image pasting performance on macOS — noticeable if you paste large screenshots often
- Improved `/effort` to work while Claude is responding — nice QoL, matches how `/model` already works

Be aggressive about skipping minor or internal changes. Only surface stuff that a daily user would notice or care about.

End each version's summary with a brief impression (1 sentence) of that release.

Use Slack-compatible formatting: *bold* (single asterisk), _italic_ (underscores), `code` (backticks). Use bullet points for individual changes. Do NOT use markdown headings (##) or double asterisks (**bold**).

Output ONLY a JSON object (no markdown fences, no extra text) with this exact structure:
{\"summaries\": [{\"version\": \"x.y.z\", \"text\": \"...markdown summary...\"}]}";

/// Resolve the prompt file path (next to the binary).
fn prompt_path() -> Result<PathBuf, SummarizeError> {
    let exe = std::env::current_exe().map_err(SummarizeError::NotFound)?;
    let dir = exe.parent().ok_or_else(|| {
        SummarizeError::NotFound(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "could not resolve binary directory",
        ))
    })?;
    Ok(dir.join("prompt.txt"))
}

/// Read prompt from file, creating it with default content if missing.
fn load_prompt() -> Result<String, SummarizeError> {
    let path = prompt_path()?;
    match std::fs::read_to_string(&path) {
        Ok(contents) => Ok(contents),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            std::fs::write(&path, DEFAULT_PROMPT).map_err(SummarizeError::PromptFile)?;
            Ok(DEFAULT_PROMPT.to_string())
        }
        Err(e) => Err(SummarizeError::PromptFile(e)),
    }
}

/// Build the full prompt string from the template and changelog entries.
pub fn build_prompt(prompt_template: &str, entries: &[VersionEntry]) -> String {
    let mut changelog = String::new();
    for entry in entries {
        changelog.push_str(&format!("## {}\n\n{}\n\n", entry.version, entry.content));
    }

    format!("{prompt_template}\n\n<changelog>\n{changelog}</changelog>")
}

#[derive(Deserialize)]
struct LlmResponse {
    summaries: Vec<VersionSummary>,
}

#[derive(Deserialize)]
struct VersionSummary {
    version: Version,
    text: String,
}

/// Match LLM response summaries to input entries, preserving input order.
fn match_summaries(
    entries: &[VersionEntry],
    response: LlmResponse,
) -> Result<Vec<Summary>, SummarizeError> {
    let mut result = Vec::with_capacity(entries.len());
    for entry in entries {
        let matched = response
            .summaries
            .iter()
            .find(|s| s.version == entry.version);
        match matched {
            Some(vs) => result.push(Summary {
                text: vs.text.clone(),
            }),
            None => {
                return Err(SummarizeError::EmptyOutput);
            }
        }
    }
    Ok(result)
}

pub struct ClaudeSummarizer {
    prompt_template: String,
}

impl ClaudeSummarizer {
    pub fn new() -> Result<Self, SummarizeError> {
        let prompt_template = load_prompt()?;
        Ok(Self { prompt_template })
    }
}

impl ChangelogSummarizer for ClaudeSummarizer {
    fn summarize(&self, entries: &[VersionEntry]) -> Result<Vec<Summary>, SummarizeError> {
        let full_prompt = build_prompt(&self.prompt_template, entries);

        let output = Command::new("claude")
            .arg("-p")
            .arg(&full_prompt)
            .output()
            .map_err(SummarizeError::NotFound)?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            return Err(SummarizeError::ProcessFailed {
                status: output.status.code().unwrap_or(-1),
                stderr,
            });
        }

        let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if text.is_empty() {
            return Err(SummarizeError::EmptyOutput);
        }

        let response: LlmResponse = serde_json::from_str(&text)?;
        match_summaries(entries, response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::version::Version;

    fn sample_entries() -> Vec<VersionEntry> {
        vec![
            VersionEntry {
                version: Version {
                    major: 2,
                    minor: 1,
                    patch: 78,
                },
                content: "- Added feature X\n- Fixed bug Y".to_string(),
            },
            VersionEntry {
                version: Version {
                    major: 2,
                    minor: 1,
                    patch: 77,
                },
                content: "- Improved performance".to_string(),
            },
        ]
    }

    #[test]
    fn build_prompt_includes_template_and_entries() {
        let entries = sample_entries();
        let result = build_prompt("Summarize this:", &entries);
        assert!(result.starts_with("Summarize this:"));
        assert!(result.contains("<changelog>"));
        assert!(result.contains("## 2.1.78"));
        assert!(result.contains("Added feature X"));
        assert!(result.contains("## 2.1.77"));
        assert!(result.contains("</changelog>"));
    }

    #[test]
    fn build_prompt_empty_entries() {
        let result = build_prompt("Summarize:", &[]);
        assert!(result.contains("<changelog>"));
        assert!(result.contains("</changelog>"));
    }

    #[test]
    fn build_prompt_includes_json_instruction() {
        let result = build_prompt(DEFAULT_PROMPT, &sample_entries());
        assert!(result.contains("JSON"));
        assert!(result.contains("summaries"));
    }

    #[test]
    fn match_summaries_maps_by_version() {
        let entries = sample_entries();
        let response = LlmResponse {
            summaries: vec![
                VersionSummary {
                    version: "2.1.77".parse().unwrap(),
                    text: "perf stuff".to_string(),
                },
                VersionSummary {
                    version: "2.1.78".parse().unwrap(),
                    text: "feature stuff".to_string(),
                },
            ],
        };

        let result = match_summaries(&entries, response).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].text, "feature stuff");
        assert_eq!(result[1].text, "perf stuff");
    }

    #[test]
    fn match_summaries_missing_version_returns_error() {
        let entries = sample_entries();
        let response = LlmResponse {
            summaries: vec![VersionSummary {
                version: "2.1.78".parse().unwrap(),
                text: "only one".to_string(),
            }],
        };

        assert!(match_summaries(&entries, response).is_err());
    }

    #[test]
    fn parse_valid_llm_json() {
        let json = r#"{"summaries": [{"version": "2.1.78", "text": "- Feature X"}]}"#;
        let response: LlmResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.summaries.len(), 1);
        assert_eq!(response.summaries[0].version.to_string(), "2.1.78");
        assert_eq!(response.summaries[0].text, "- Feature X");
    }

    #[test]
    fn parse_malformed_json_returns_error() {
        let json = "not json at all";
        let result = serde_json::from_str::<LlmResponse>(json);
        assert!(result.is_err());
    }
}
