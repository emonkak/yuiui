use rust_ui_derive::WidgetMeta;
use std::mem;
use std::sync::Arc;

use crate::event::{EventHandler, HandlerId};
use crate::lifecycle::Lifecycle;
use crate::paint::PaintContext;

use super::element::Children;
use super::{Widget, WidgetMeta};

#[derive(Debug, WidgetMeta)]
pub struct Subscriber<Handle: 'static> {
    handlers: Vec<Arc<dyn EventHandler<Handle> + Send + Sync>>,
}

#[derive(Default)]
pub struct SubscriberState {
    registered_handler_ids: Vec<HandlerId>,
}

impl<Handle> Subscriber<Handle> {
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
        }
    }

    pub fn on<Handler>(mut self, handler: Handler) -> Self
    where
        Handler: EventHandler<Handle> + Send + Sync + 'static,
    {
        self.handlers.push(Arc::new(handler));
        self
    }
}

impl<Handle> Widget<Handle> for Subscriber<Handle> {
    type State = SubscriberState;

    fn should_update(
        &self,
        new_widget: &Self,
        old_children: &Children<Handle>,
        new_children: &Children<Handle>,
        _state: &Self::State,
    ) -> bool {
        !Arc::ptr_eq(&old_children, &new_children) || self.handlers != new_widget.handlers
    }

    #[inline]
    fn lifecycle(
        &self,
        lifecycle: Lifecycle<&Self, &Children<Handle>>,
        state: &mut Self::State,
        context: &mut PaintContext<Handle>,
    ) {
        match lifecycle {
            Lifecycle::OnMount(_) => {
                for handler in self.handlers.iter() {
                    let handler_id = context.add_handler(Arc::clone(handler));
                    state.registered_handler_ids.push(handler_id);
                }
            }
            Lifecycle::OnUpdate(new_widget, _, _) => {
                let intersected_len = self.handlers.len().min(new_widget.handlers.len());

                for index in 0..intersected_len {
                    let handler = &self.handlers[index];
                    let new_handler = &new_widget.handlers[index];

                    if handler.as_ptr() != new_handler.as_ptr() {
                        let handler_id = context.add_handler(Arc::clone(&new_handler));
                        let old_handler_id =
                            mem::replace(&mut state.registered_handler_ids[index], handler_id);
                        context.remove_handler(old_handler_id);
                    };
                }

                for _ in intersected_len..self.handlers.len() {
                    let old_handler_id = state.registered_handler_ids.pop().unwrap();
                    context.remove_handler(old_handler_id);
                }

                for index in intersected_len..new_widget.handlers.len() {
                    let new_handler = &new_widget.handlers[index];
                    let handler_id = context.add_handler(Arc::clone(&new_handler));
                    state.registered_handler_ids.push(handler_id);
                }
            }
            Lifecycle::OnUnmount(_) => {
                for handler_id in mem::take(&mut state.registered_handler_ids) {
                    context.remove_handler(handler_id);
                }
            }
        }
    }
}
