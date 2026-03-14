use crate::formatter::Message;

#[derive(Debug, thiserror::Error)]
pub enum PublishError {
    #[error("output failed: {0}")]
    Output(#[from] std::io::Error),
}

pub trait MessagePublisher {
    fn publish(&self, message: &Message) -> Result<(), PublishError>;
}
