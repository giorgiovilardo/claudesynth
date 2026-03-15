mod edit;
mod editor;
mod run;
mod show;

pub use edit::edit;
pub use editor::{EditError, EnvEditor};
pub use run::run;
pub use show::show;

#[derive(Debug, clap::Subcommand)]
pub enum Command {
    /// Fetch new changelog versions, summarize, and publish
    Run,
    /// Show the stored summary for a specific version
    Show {
        /// Version number (e.g. 2.1.78)
        version: crate::version::Version,
    },
    /// Edit the stored summary for a version in $EDITOR
    Edit {
        /// Version number (e.g. 2.1.78)
        version: crate::version::Version,
    },
}
