use glib::Sender;
use gtk::Application as GtkApplication;
use std::any::Any;
use yuiui::EventDestination;

#[derive(Debug)]
pub struct Backend {
    application: GtkApplication,
    event_port: Sender<(Box<dyn Any + Send>, EventDestination)>,
}

impl Backend {
    pub(super) fn new(
        application: GtkApplication,
        event_port: Sender<(Box<dyn Any + Send>, EventDestination)>,
    ) -> Self {
        Self {
            application,
            event_port,
        }
    }

    pub fn application(&self) -> &GtkApplication {
        &self.application
    }

    pub fn dispatch_event(&self, event: Box<dyn Any + Send>, destination: EventDestination) {
        let port = self.event_port.clone();
        port.send((event, destination)).unwrap();
    }
}
