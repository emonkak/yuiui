use std::marker::PhantomData;

use crate::widget::element::ElementId;
use crate::widget::message::{Message, MessageSender};

pub struct OutboundEmitter<'a, Outbound> {
    message_sender: &'a MessageSender,
    outbound_queue: Vec<Outbound>,
    outbound_type: PhantomData<Outbound>,
}

pub struct InboundEmitter<'a, Inbound> {
    origin_id: ElementId,
    message_sender: &'a MessageSender,
    inbound_type: PhantomData<Inbound>,
}

impl<'a, Outbound> OutboundEmitter<'a, Outbound> {
    pub fn new(message_sender: &'a MessageSender) -> Self {
        Self {
            message_sender,
            outbound_queue: Vec::new(),
            outbound_type: PhantomData,
        }
    }

    pub fn create_inbound_emitter<Inbound>(&self, listener_id: ElementId) -> InboundEmitter<Inbound> {
        InboundEmitter::new(listener_id, &self.message_sender)
    }

    pub fn outbound_events(&self) -> &Vec<Outbound> {
        &self.outbound_queue
    }

    pub fn emit(&mut self, event: Outbound)
    where
        Outbound: Send + 'static
    {
        self.outbound_queue.push(event)
    }

    pub fn broadcast(&mut self, event: Outbound)
    where
        Outbound: Send + 'static,
    {
        self.message_sender
            .send(Message::Broadcast(Box::new(event)))
            .unwrap();
    }
}

impl<'a, Inbound> InboundEmitter<'a, Inbound> {
    pub fn new(origin_id: ElementId, message_sender: &'a MessageSender) -> Self {
        Self {
            origin_id,
            message_sender,
            inbound_type: PhantomData,
        }
    }

    pub fn emit(&mut self, event: Inbound)
    where
        Inbound: Send + 'static
    {
        self.message_sender
            .send(Message::Send(self.origin_id, Box::new(event)))
            .unwrap();
    }
}
