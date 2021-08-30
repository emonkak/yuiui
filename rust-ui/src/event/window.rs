use crate::geometrics::PhysicalSize;

use super::EventType;

#[derive(Debug)]
pub struct WindowCloseEvent;

pub struct WindowClose;

impl EventType for WindowClose {
    type Event = WindowCloseEvent;
}

#[derive(Debug)]
pub struct WindowResizeEvent {
    pub size: PhysicalSize,
}

pub struct WindowResize;

impl EventType for WindowResize {
    type Event = WindowResizeEvent;
}
