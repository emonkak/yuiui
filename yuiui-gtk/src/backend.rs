use gtk::glib;
use std::sync::mpsc;
use yuiui::{IdPathBuf, TransferableEvent};

#[derive(Debug)]
pub struct Backend {
    window: gtk::Window,
    event_port: EventPort,
}

impl Backend {
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

#[derive(Debug, Clone)]
pub struct EventPort {
    event_sender: glib::Sender<TransferableEvent>,
}

impl EventPort {
    pub fn new(event_sender: glib::Sender<TransferableEvent>) -> Self {
        Self { event_sender }
    }

    pub fn forward<T: Send + 'static>(
        &self,
        destination: IdPathBuf,
        payload: T,
    ) -> Result<(), mpsc::SendError<TransferableEvent>> {
        let event = TransferableEvent::Forward(destination, Box::new(payload));
        self.event_sender.send(event)
    }

    pub fn broadcast<T: Send + 'static>(
        &self,
        destinations: Vec<IdPathBuf>,
        payload: T,
    ) -> Result<(), mpsc::SendError<TransferableEvent>> {
        let event = TransferableEvent::Broadcast(destinations, Box::new(payload));
        self.event_sender.send(event)
    }
}
