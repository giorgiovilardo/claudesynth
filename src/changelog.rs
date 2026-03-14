mod domain;
mod http;
mod parser;

pub use domain::{ChangelogError, ChangelogProvider, VersionEntry};
pub use http::HttpChangelog;
