use std::any::Any;
use std::mem;
use std::sync::Arc;

use rust_ui_derive::WidgetMeta;

use crate::event::{EventHandler, EventHandlerId, EventType};
use crate::paint::{Lifecycle, PaintContext};

use super::element::Children;
use super::state::StateCell;
use super::widget::{Widget, WidgetMeta};

#[derive(WidgetMeta)]
pub struct Subscriber {
    handlers: Vec<Arc<EventHandler>>,
}

#[derive(Default)]
pub struct SubscriberState {
    registered_handler_ids: Vec<EventHandlerId>,
}

impl Subscriber {
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
        }
    }

    pub fn on<EventType, F>(mut self, event_type: EventType, callback: F) -> Self
    where
        EventType: self::EventType + 'static,
        F: Fn(&EventType::Event) + Send + Sync + 'static,
    {
        let handler = EventHandler::new(event_type.type_id(), callback);
        self.handlers.push(Arc::new(handler));
        self
    }
}

impl<Renderer> Widget<Renderer> for Subscriber {
    type State = SubscriberState;

    #[inline]
    fn lifecycle(
        self: Arc<Self>,
        _children: Children<Renderer>,
        state: StateCell<Self::State>,
        lifecycle: Lifecycle<Arc<Self>, Children<Renderer>>,
        _renderer: &mut Renderer,
        context: &mut PaintContext,
    ) {
        match lifecycle {
            Lifecycle::DidMount() => {
                let registered_handler_ids = self
                    .handlers
                    .iter()
                    .map(|handler| context.add_handler(handler.clone()))
                    .collect();
                state.borrow().registered_handler_ids = registered_handler_ids;
            }
            Lifecycle::DidUpdate(_, _) => {
                let registered_handler_ids = self
                    .handlers
                    .iter()
                    .map(|handler| context.add_handler(handler.clone()))
                    .collect();
                let old_registered_handler_ids = mem::replace(
                    &mut state.borrow().registered_handler_ids,
                    registered_handler_ids,
                );
                for handler_id in old_registered_handler_ids {
                    context.remove_handler(handler_id);
                }
            }
            Lifecycle::DidUnmount() => {
                let old_registered_handler_ids =
                    mem::take(&mut state.borrow().registered_handler_ids);
                for handler_id in old_registered_handler_ids {
                    context.remove_handler(handler_id);
                }
            }
        }
    }
}
