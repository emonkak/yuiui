use std::mem;
use std::sync::Arc;

use rust_ui_derive::WidgetMeta;

use crate::event::{EventHandler, HandlerId};
use crate::paint::{PaintContext, PaintCycle};

use super::element::Children;
use super::{Widget, WidgetMeta};

#[derive(Debug, WidgetMeta)]
pub struct Subscriber<Painter: 'static> {
    handlers: Vec<Arc<dyn EventHandler<Painter>>>,
}

#[derive(Default)]
pub struct SubscriberState {
    registered_handler_ids: Vec<HandlerId>,
}

impl<Painter> Subscriber<Painter> {
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
        }
    }

    pub fn on<Handler>(mut self, handler: Handler) -> Self
    where
        Handler: EventHandler<Painter> + 'static,
    {
        self.handlers.push(Arc::new(handler));
        self
    }
}

impl<Painter> Widget<Painter> for Subscriber<Painter> {
    type State = SubscriberState;

    fn should_update(
        &self,
        new_widget: &Self,
        old_children: &Children<Painter>,
        new_children: &Children<Painter>,
        _state: &Self::State,
    ) -> bool {
        !Arc::ptr_eq(&old_children, &new_children) || self.handlers != new_widget.handlers
    }

    #[inline]
    fn on_paint_cycle(
        &self,
        paint_cycle: PaintCycle<&Self, &Children<Painter>>,
        state: &mut Self::State,
        _painter: &mut Painter,
        context: &mut PaintContext<Painter>,
    ) {
        match paint_cycle {
            PaintCycle::DidMount(_children) => {
                for handler in self.handlers.iter() {
                    let handler_id = context.add_handler(Arc::clone(handler));
                    state.registered_handler_ids.push(handler_id);
                }
            }
            PaintCycle::DidUpdate(_old_children, new_widget, _new_children) => {
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
            PaintCycle::DidUnmount(_children) => {
                for handler_id in mem::take(&mut state.registered_handler_ids) {
                    context.remove_handler(handler_id);
                }
            }
        }
    }
}
