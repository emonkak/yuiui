use std::any::Any;
use std::borrow::{Borrow};
use std::marker::PhantomData;
use std::sync::mpsc::Sender;

use super::element::ElementId;

#[derive(Debug, Clone)]
pub struct MessageSink<Message, Sender> {
    element_id: ElementId,
    version: usize,
    message_sender: Sender,
    message_type: PhantomData<Message>,
}

#[derive(Debug)]
pub struct MessageContext<Message> {
    pub element_id: ElementId,
    pub version: usize,
    message_type: PhantomData<Message>,
}

pub type MessageSender = Sender<(ElementId, usize, AnyMessage)>;

pub type AnyMessage = Box<dyn Any + Send>;

impl<Message, Sender> MessageSink<Message, Sender> {
    pub fn new(element_id: ElementId, version: usize, message_sender: Sender) -> Self {
        Self {
            element_id,
            version,
            message_sender,
            message_type: PhantomData,
        }
    }

    pub fn send(&self, message: Message)
    where
        Sender: Borrow<MessageSender>,
        Message: Send + 'static,
    {
        self.message_sender
            .borrow()
            .send((self.element_id, self.version, Box::new(message)))
            .unwrap();
    }

    pub fn share(&self) -> MessageSink<Message, MessageSender>
    where
        Sender: Borrow<MessageSender>,
        Message: Send + 'static,
    {
        MessageSink {
            element_id: self.element_id,
            version: self.version,
            message_sender: self.message_sender.borrow().clone(),
            message_type: self.message_type,
        }
    }
}

impl<Message> MessageContext<Message> {
    pub fn new(element_id: ElementId, version: usize) -> Self {
        Self {
            element_id,
            version,
            message_type: PhantomData,
        }
    }
}

impl<Message> Clone for MessageContext<Message> {
    fn clone(&self) -> Self {
        Self {
            element_id: self.element_id,
            version: self.version,
            message_type: self.message_type,
        }
    }
}

impl<Message> Copy for MessageContext<Message> {}
