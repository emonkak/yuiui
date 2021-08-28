use std::mem;
use std::sync::Arc;

use rust_ui_derive::WidgetMeta;

use crate::event::{EventListener, EventListenerId, EventType};
use crate::paint::{Lifecycle, PaintContext};

use super::element::Children;
use super::message::{MessageSender, MessageSink, MessageContext};
use super::{Widget, WidgetMeta};

#[derive(Debug, WidgetMeta)]
pub struct Subscriber {
    listeners: Vec<Arc<EventListener>>,
}

#[derive(Default)]
pub struct SubscriberState {
    registered_listener_ids: Vec<EventListenerId>,
}

impl Subscriber {
    pub fn new() -> Self {
        Self {
            listeners: Vec::new(),
        }
    }

    pub fn on<Message, EventType, ListenerFn>(
        mut self,
        message_context: MessageContext<Message>,
        event_type: EventType,
        listener_fn: ListenerFn,
    ) -> Self
    where
        Message: Send + Sync + 'static,
        EventType: self::EventType + 'static,
        EventType::Event: 'static,
        ListenerFn:
            Fn(&EventType::Event, MessageSink<Message, &MessageSender>) + Send + Sync + 'static,
    {
        let listener = EventListener::new(message_context.element_id, event_type, listener_fn);
        self.listeners.push(Arc::new(listener));
        self
    }
}

impl<Renderer> Widget<Renderer> for Subscriber {
    type State = ();

    type Message = ();

    type PaintObject = SubscriberState;

    #[inline]
    fn lifecycle(
        &self,
        _children: &Children<Renderer>,
        paint_object: &mut Self::PaintObject,
        lifecycle: Lifecycle<Arc<Self>, Children<Renderer>>,
        _renderer: &mut Renderer,
        context: &mut PaintContext,
    ) {
        match lifecycle {
            Lifecycle::DidMount() => {
                for listener in self.listeners.iter() {
                    let listener_id = context.add_listener(listener.clone());
                    paint_object.registered_listener_ids.push(listener_id);
                }
            }
            Lifecycle::DidUpdate(_old_widget, _old_children) => {
                for listener_id in mem::take(&mut paint_object.registered_listener_ids) {
                    context.remove_listener(listener_id);
                }

                for listener in self.listeners.iter() {
                    let listener_id = context.add_listener(listener.clone());
                    paint_object.registered_listener_ids.push(listener_id);
                }
            }
            Lifecycle::DidUnmount() => {
                for listener_id in mem::take(&mut paint_object.registered_listener_ids) {
                    context.remove_listener(listener_id);
                }
            }
        }
    }
}
