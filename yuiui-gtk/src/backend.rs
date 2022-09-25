use gtk::glib;
use std::any::Any;
use yuiui::EventDestination;

#[derive(Debug)]
pub struct GtkBackend {
    application: gtk::Application,
    window: gtk::ApplicationWindow,
    event_port: glib::Sender<(Box<dyn Any + Send>, EventDestination)>,
}

impl GtkBackend {
    pub(super) fn new(
        application: gtk::Application,
        window: gtk::ApplicationWindow,
        event_port: glib::Sender<(Box<dyn Any + Send>, EventDestination)>,
    ) -> Self {
        Self {
            application,
            window,
            event_port,
        }
    }

    pub fn application(&self) -> &gtk::Application {
        &self.application
    }

    pub fn window(&self) -> &gtk::ApplicationWindow {
        &self.window
    }

    pub fn event_port(&self) -> glib::Sender<(Box<dyn Any + Send>, EventDestination)> {
        self.event_port.clone()
    }
}
