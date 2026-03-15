use std::io::Write;
use std::process::ExitStatus;

#[derive(Debug, thiserror::Error)]
pub enum EditError {
    #[error("editor exited with {0}")]
    EditorFailed(ExitStatus),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, PartialEq)]
pub enum EditOutcome {
    Changed(String),
    Unchanged,
}

pub trait Editor {
    fn edit(&self, initial_content: &str) -> Result<EditOutcome, EditError>;
}

pub struct EnvEditor;

impl EnvEditor {
    fn resolve_editor() -> String {
        std::env::var("EDITOR")
            .or_else(|_| std::env::var("VISUAL"))
            .unwrap_or_else(|_| "vim".to_string())
    }
}

impl Editor for EnvEditor {
    fn edit(&self, initial_content: &str) -> Result<EditOutcome, EditError> {
        let editor = Self::resolve_editor();

        let mut tmp = tempfile::Builder::new()
            .prefix("claudesynth-")
            .suffix(".md")
            .tempfile()?;

        tmp.write_all(initial_content.as_bytes())?;
        tmp.flush()?;

        let path = tmp.path().to_owned();

        let status = std::process::Command::new(&editor).arg(&path).status()?;

        if !status.success() {
            return Err(EditError::EditorFailed(status));
        }

        let new_content = std::fs::read_to_string(&path)?;

        if new_content.trim_end() == initial_content.trim_end() {
            Ok(EditOutcome::Unchanged)
        } else {
            Ok(EditOutcome::Changed(new_content))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockEditor {
        outcome: Result<EditOutcome, &'static str>,
    }

    impl Editor for MockEditor {
        fn edit(&self, _initial_content: &str) -> Result<EditOutcome, EditError> {
            match &self.outcome {
                Ok(EditOutcome::Changed(s)) => Ok(EditOutcome::Changed(s.clone())),
                Ok(EditOutcome::Unchanged) => Ok(EditOutcome::Unchanged),
                Err(msg) => Err(EditError::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    *msg,
                ))),
            }
        }
    }

    #[test]
    fn mock_editor_returns_changed() {
        let editor = MockEditor {
            outcome: Ok(EditOutcome::Changed("new content".to_string())),
        };
        let result = editor.edit("old content").unwrap();
        assert_eq!(result, EditOutcome::Changed("new content".to_string()));
    }

    #[test]
    fn mock_editor_returns_unchanged() {
        let editor = MockEditor {
            outcome: Ok(EditOutcome::Unchanged),
        };
        let result = editor.edit("content").unwrap();
        assert_eq!(result, EditOutcome::Unchanged);
    }

    #[test]
    fn mock_editor_returns_error() {
        let editor = MockEditor {
            outcome: Err("no editor"),
        };
        let result = editor.edit("content");
        assert!(result.is_err());
    }
}
