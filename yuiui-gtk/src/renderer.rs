use gtk::glib;
use std::any::Any;
use yuiui::EventDestination;

pub type EventPort = glib::Sender<(Box<dyn Any + Send>, EventDestination)>;

#[derive(Debug)]
pub struct GtkRenderer {
    window: gtk::Window,
    event_port: EventPort,
}

impl GtkRenderer {
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
