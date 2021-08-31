use std::any::Any;
use std::marker::PhantomData;

use crate::event::MouseDown;

use super::element::{ElementId};
use super::message::{MessageQueue, Message};
use super::widget::{AsAny, Widget};

#[derive(Debug)]
pub struct MouseDownBehavior<Child: 'static, SelectorFn: 'static, Outbound: 'static> {
    child: Child,
    listener_id: ElementId,
    selector_fn: SelectorFn,
    outbound_type: PhantomData<Outbound>,
}

impl<Child, SelectorFn, Outbound> MouseDownBehavior<Child, SelectorFn, Outbound> {
    pub fn new(child: Child, listener_id: ElementId, selector_fn: SelectorFn) -> Self {
        Self {
            child,
            listener_id,
            selector_fn,
            outbound_type: PhantomData,
        }
    }
}

impl<Child, SelectorFn, Outbound, Renderer> Widget<Renderer> for MouseDownBehavior<Child, SelectorFn, Outbound>
where
    Child: Widget<Renderer>,
    SelectorFn: Fn(&MouseDown) -> Outbound + Send + Sync + 'static,
    Outbound: Send + Sync + 'static,
    Renderer: 'static,
{
    type State = ();
    type Message = MouseDown;

    fn update(
        &self,
        _state: &mut Self::State,
        message: &Self::Message,
        message_queue: &mut MessageQueue,
    ) -> bool {
        let outbound_message = (self.selector_fn)(message);
        let message = self::Message::Send(self.listener_id, Box::new(outbound_message));
        message_queue.enqueue(message);
        false
    }
}

impl<Child, SelectorFn, Outbound> AsAny for MouseDownBehavior<Child, SelectorFn, Outbound> {
    fn as_any(&self) -> &dyn Any {
       self
    }
}
