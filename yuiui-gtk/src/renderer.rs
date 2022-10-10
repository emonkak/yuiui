use gtk::glib;
use std::sync::mpsc;
use yuiui::{Event, IdPathBuf};

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

#[derive(Debug, Clone)]
pub struct EventPort {
    event_sender: glib::Sender<Event>,
}

impl EventPort {
    pub fn new(event_sender: glib::Sender<Event>) -> Self {
        Self { event_sender }
    }

    pub fn forward<T: Send + 'static>(
        &self,
        destination: IdPathBuf,
        payload: T,
    ) -> Result<(), mpsc::SendError<Event>> {
        let event = Event::Forward(destination, Box::new(payload));
        self.event_sender.send(event)
    }

    pub fn broadcast<T: Send + 'static>(
        &self,
        destinations: Vec<IdPathBuf>,
        payload: T,
    ) -> Result<(), mpsc::SendError<Event>> {
        let event = Event::Broadcast(destinations, Box::new(payload));
        self.event_sender.send(event)
    }
}
