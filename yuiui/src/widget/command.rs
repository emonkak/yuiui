use std::future::Future;
use std::pin::Pin;
use std::time::Instant;

pub enum Command<Message> {
    QuitApplication,
    RequestUpdate,
    Send(Message),
    Perform(Pin<Box<dyn Future<Output = Message>>>),
    RequestIdle(Box<dyn FnOnce(Instant) -> Message>),
}
