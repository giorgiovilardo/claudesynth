use crate::changelog::VersionEntry;
use crate::summarizer::Summary;

#[derive(Debug, thiserror::Error)]
pub enum FormatError {
    #[error("no entries provided")]
    NoEntries,
}

#[derive(Debug, Clone)]
pub struct Message {
    pub text: String,
}

pub trait SummaryFormatter {
    fn format(&self, items: &[(VersionEntry, Summary)]) -> Result<Message, FormatError>;
}
