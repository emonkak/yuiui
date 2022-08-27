use std::any::Any;

use crate::component_node::ComponentStack;
use crate::effect::{EffectContext, EffectContextVisitor};
use crate::event::Event;
use crate::state::State;
use crate::view::View;
use crate::widget::{Widget, WidgetEvent};

use super::{WidgetNode, WidgetState};

pub struct InternalEventVisitor<'a> {
    event: &'a dyn Any,
}

impl<'a> InternalEventVisitor<'a> {
    pub fn new(event: &'a dyn Any) -> Self {
        Self { event }
    }
}

impl<'a> EffectContextVisitor for InternalEventVisitor<'a> {
    fn visit<V, CS, S, E>(
        &mut self,
        node: &mut WidgetNode<V, CS, S, E>,
        state: &S,
        env: &E,
        context: &mut EffectContext<S>,
    ) where
        V: View<S, E>,
        CS: ComponentStack<S, E, View = V>,
        S: State,
    {
        match node.state.as_mut().unwrap() {
            WidgetState::Prepared(widget, _) | WidgetState::Dirty(widget, _) => {
                let event = <V::Widget as WidgetEvent>::Event::from_any(self.event)
                    .expect("cast any event to widget event");
                let result = widget.event(event, &node.children, context.id_path(), state, env);
                context.process_result(result);
            }
            WidgetState::Uninitialized(_) => {}
        }
    }
}
