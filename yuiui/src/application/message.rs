use std::time::Instant;

#[derive(Debug)]
pub enum ApplicationMessage<Message> {
    Quit,
    Render(Instant),
    Broadcast(Message),
}
