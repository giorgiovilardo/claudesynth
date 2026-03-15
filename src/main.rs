mod changelog;
mod commands;
mod formatter;
mod history;
mod publisher;
mod summarizer;
mod version;

use clap::Parser;

use crate::changelog::HttpChangelog;
use crate::formatter::MarkdownSummaryFormatter;
use crate::history::JsonHistoryRepository;
use crate::publisher::StdoutMessagePublisher;
use crate::summarizer::ClaudeSummarizer;
use crate::version::Version;

fn main() {
    if let Err(err) = run() {
        print_error(&err);
        std::process::exit(1);
    }
}

fn run() -> Result<(), AppError> {
    let cli = Cli::parse();
    let repo = JsonHistoryRepository::default_location().map_err(AppError::HistoryLocation)?;

    match cli.command {
        commands::Command::Run => {
            let summarizer = ClaudeSummarizer::new()?;
            commands::run(
                &HttpChangelog,
                &summarizer,
                &MarkdownSummaryFormatter,
                &StdoutMessagePublisher,
                &repo,
            )
        }
        commands::Command::Show { version } => commands::show(&version, &repo),
        commands::Command::Edit { version } => {
            commands::edit(&version, &repo, &commands::EnvEditor)
        }
    }
}

#[derive(Parser, Debug)]
#[command(
    name = "claudesynth",
    version,
    about = "Claude Code changelog summary generator"
)]
struct Cli {
    #[command(subcommand)]
    command: commands::Command,
}

#[derive(Debug, thiserror::Error)]
enum AppError {
    #[error("Failed to resolve history file location")]
    HistoryLocation(#[source] history::HistoryError),
    #[error("Failed to load history: {0}")]
    HistoryLoad(#[source] history::HistoryError),
    #[error("Failed to save history: {0}")]
    HistorySave(#[source] history::HistoryError),
    #[error("Failed to fetch changelog: {0}")]
    Changelog(#[from] changelog::ChangelogError),
    #[error("Failed to format message: {0}")]
    Format(#[from] formatter::FormatError),
    #[error("Failed to publish message: {0}")]
    Publish(#[from] publisher::PublishError),
    #[error("{0}")]
    Summarize(#[from] summarizer::SummarizeError),
    #[error("{0}")]
    Editor(#[from] commands::EditError),
    #[error("Version {0} not found in history")]
    VersionNotFound(Version),
}

fn print_error(err: &AppError) {
    match err {
        AppError::Summarize(summarizer::SummarizeError::NotFound(_)) => {
            eprintln!(
                "Error: claude CLI not found. Install it from https://docs.anthropic.com/en/docs/claude-code"
            );
        }
        AppError::Summarize(summarizer::SummarizeError::EmptyOutput) => {
            eprintln!("Error: claude returned empty output. Try running again.");
        }
        AppError::Changelog(changelog::ChangelogError::Http(_)) => {
            eprintln!("Error: could not reach GitHub. Check your network connection.");
        }
        AppError::HistoryLoad(history::HistoryError::Json(_)) => {
            eprintln!(
                "Error: history file is corrupted. Delete claudesynth-history.json and retry."
            );
        }
        AppError::VersionNotFound(v) => {
            eprintln!("Error: version {v} not found in history. Run `claudesynth run` first.");
        }
        AppError::Editor(commands::EditError::NoEditor) => {
            eprintln!("Error: no editor configured. Set $EDITOR or $VISUAL.");
        }
        _ => eprintln!("Error: {err}"),
    }
}

#[cfg(test)]
mod test_helpers {
    use crate::changelog::VersionEntry;
    use crate::summarizer::Summary;
    use crate::version::Version;

    pub fn version_entry(version: &str, content: &str) -> VersionEntry {
        VersionEntry {
            version: version.parse::<Version>().unwrap(),
            content: content.to_string(),
        }
    }

    pub fn summary(text: &str) -> Summary {
        Summary {
            text: text.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn cli_has_version_flag() {
        let cmd = Cli::command();
        assert_eq!(cmd.get_version(), Some(env!("CARGO_PKG_VERSION")));
    }

    #[test]
    fn cli_parses_run_subcommand() {
        let cli = Cli::try_parse_from(["claudesynth", "run"]).unwrap();
        assert!(matches!(cli.command, commands::Command::Run));
    }

    #[test]
    fn cli_parses_edit_subcommand() {
        let cli = Cli::try_parse_from(["claudesynth", "edit", "2.1.78"]).unwrap();
        match cli.command {
            commands::Command::Edit { version } => {
                assert_eq!(version.to_string(), "2.1.78");
            }
            _ => panic!("expected Edit command"),
        }
    }

    #[test]
    fn cli_parses_show_subcommand() {
        let cli = Cli::try_parse_from(["claudesynth", "show", "2.1.78"]).unwrap();
        match cli.command {
            commands::Command::Show { version } => {
                assert_eq!(version.to_string(), "2.1.78");
            }
            _ => panic!("expected Show command"),
        }
    }
}
