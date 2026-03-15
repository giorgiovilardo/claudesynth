mod run;
mod show;

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
}
