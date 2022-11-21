use gtk::glib;
use std::sync::mpsc;
use yuiui::{IdPathBuf, TransferableEvent};

#[derive(Debug, Clone)]
pub struct Backend<E> {
    entry_point: E,
    event_port: glib::Sender<TransferableEvent>,
}

impl<E> Backend<E> {
    pub(crate) fn new(entry_point: E, event_port: glib::Sender<TransferableEvent>) -> Self {
        Self {
            entry_point,
            event_port,
        }
    }

    pub fn entry_point(&self) -> &E {
        &self.entry_point
    }

    pub fn forward_event<T: Send + 'static>(
        &self,
        destination: IdPathBuf,
        payload: T,
    ) -> Result<(), mpsc::SendError<TransferableEvent>> {
        let event = TransferableEvent::Forward(destination, Box::new(payload));
        self.event_port.send(event)
    }

    pub fn broadcast_event<T: Send + 'static>(
        &self,
        destinations: Vec<IdPathBuf>,
        payload: T,
    ) -> Result<(), mpsc::SendError<TransferableEvent>> {
        let event = TransferableEvent::Broadcast(destinations, Box::new(payload));
        self.event_port.send(event)
    }
}
