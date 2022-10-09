use gtk::glib;
use std::any::Any;
use yuiui::IdPathBuf;

#[derive(Debug)]
pub struct Renderer {
    window: gtk::Window,
    event_port: EventPort,
}

impl Renderer {
    pub(crate) fn new(window: gtk::Window, event_port: EventPort) -> Self {
        Self { window, event_port }
    }

    pub fn window(&self) -> &gtk::Window {
        &self.window
    }

    pub fn event_port(&self) -> &EventPort {
        &self.event_port
    }
}

pub type EventPort = glib::Sender<(IdPathBuf, Box<dyn Any + Send>)>;
