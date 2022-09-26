use gtk::glib;
use std::any::Any;
use yuiui::EventDestination;

#[derive(Debug)]
pub struct GtkBackend {
    window: gtk::Window,
    event_port: glib::Sender<(Box<dyn Any + Send>, EventDestination)>,
}

impl GtkBackend {
    pub(super) fn new(
        window: gtk::Window,
        event_port: glib::Sender<(Box<dyn Any + Send>, EventDestination)>,
    ) -> Self {
        Self {
            window,
            event_port,
        }
    }

    pub fn window(&self) -> &gtk::Window {
        &self.window
    }

    pub fn event_port(&self) -> glib::Sender<(Box<dyn Any + Send>, EventDestination)> {
        self.event_port.clone()
    }
}
