use crate::formatter::Message;

use super::domain::{MessagePublisher, PublishError};

pub struct StdoutMessagePublisher;

impl MessagePublisher for StdoutMessagePublisher {
    fn publish(&self, message: &Message) -> Result<(), PublishError> {
        println!("{}", message.text);
        Ok(())
    }
}
