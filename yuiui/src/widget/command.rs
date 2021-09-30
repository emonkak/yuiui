use std::future::Future;
use std::pin::Pin;
use std::time::{Duration, Instant};

pub enum Command<Message> {
    QuitApplication,
    RequestUpdate,
    Send(Message),
    Perform(Pin<Box<dyn Future<Output = Message>>>),
    Delay(Duration, Box<dyn FnOnce() -> Message>),
    RequestAnimationFrame(Box<dyn FnOnce() -> Message>),
    RequestIdle(Box<dyn FnOnce(Instant) -> Message>),
}
