use std::future::Future;
use std::time::{Duration, Instant};
use tokio::task::JoinHandle;

use crate::event::WindowEvent;

#[derive(Debug)]
pub enum Event<Message, WindowId> {
    LoopInitialized,
    Message(Message),
    WindowEvent(WindowId, WindowEvent),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ControlFlow {
    Continue,
    Break,
}

pub trait EventLoop<Message> {
    type WindowId;

    type Context: EventLoopContext<Message>;

    fn run<F>(&mut self, callback: F) -> anyhow::Result<()>
    where
        F: FnMut(Event<Message, Self::WindowId>, &Self::Context) -> ControlFlow;
}

pub trait EventLoopContext<Message> {
    fn send(&self, message: Message);

    fn perform<F>(&self, future: F) -> JoinHandle<()>
    where
        F: 'static + Future<Output = Message>;

    fn delay<F>(&self, duration: Duration, callback: F) -> JoinHandle<()>
    where
        F: 'static + FnOnce() -> Message;

    fn request_animation_frame<F>(&self, callback: F) -> JoinHandle<()>
    where
        F: 'static + FnOnce() -> Message;

    fn request_idle<F>(&self, callback: F) -> JoinHandle<()>
    where
        F: 'static + FnOnce(Instant) -> Message;
}
