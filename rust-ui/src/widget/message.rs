use std::any::Any;

use std::collections::VecDeque;

use super::element::ElementId;

#[derive(Debug)]
pub enum Message {
    Broadcast(Box<dyn Any + Send>),
    Send(ElementId, Box<dyn Any + Send>),
}

#[derive(Debug)]
pub struct MessageSink {
    element_id: ElementId,
    message_queue: VecDeque<Message>,
}

impl MessageSink {
    pub fn new(element_id: ElementId) -> Self {
        Self {
            element_id,
            message_queue: VecDeque::new(),
        }
    }

    pub fn enqueue(&mut self, message: Message) {
        self.message_queue.push_back(message);
    }

    pub fn dequeue(&mut self) -> Option<Message> {
        self.message_queue.pop_front()
    }
}
