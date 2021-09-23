use std::error::Error;
use std::future::Future;
use std::time::Instant;
use tokio::task::JoinHandle;

use super::event::Event;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ControlFlow {
    Continue,
    Break,
}

pub trait EventLoop<Message> {
    type WindowId;

    type Context: EventLoopContext<Message>;

    fn run<F>(&mut self, callback: F) -> Result<(), Box<dyn Error>>
    where
        F: FnMut(Event<Message, Self::WindowId>, &Self::Context) -> ControlFlow;
}

pub trait EventLoopContext<Message> {
    fn send(&self, message: Message);

    fn perform<F>(&self, future: F) -> JoinHandle<()>
    where
        F: 'static + Future<Output = Message>;

    fn request_idle<F>(&self, callback: F) -> JoinHandle<()>
    where
        F: 'static + FnOnce(Instant) -> Message;
}
