use std::any::Any;

use crate::component_node::ComponentStack;
use crate::context::{EffectContext, IdContext};
use crate::event::Event;
use crate::sequence::{TraversableSeq, TraversableSeqVisitor};
use crate::state::State;
use crate::view::View;
use crate::widget::{Widget, WidgetEvent};

use super::{WidgetNode, WidgetState};

pub struct EventVisitor<'a> {
    event: &'a dyn Any,
    result: bool,
}

impl<'a> EventVisitor<'a> {
    pub fn new(event: &'a dyn Any) -> Self {
        Self {
            event,
            result: false,
        }
    }

    pub fn result(&self) -> bool {
        self.result
    }
}

impl<'a, V, CS, S, E> TraversableSeqVisitor<WidgetNode<V, CS, S, E>, EffectContext<S>, S, E>
    for EventVisitor<'a>
where
    V: View<S, E>,
    CS: ComponentStack<S, E, View = V>,
    S: State,
{
    fn visit(
        &mut self,
        node: &mut WidgetNode<V, CS, S, E>,
        state: &S,
        env: &E,
        context: &mut EffectContext<S>,
    ) {
        match node.state.as_mut().unwrap() {
            WidgetState::Prepared(widget, _) | WidgetState::Dirty(widget, _) => {
                if node.event_mask.contains(&self.event.type_id()) {
                    node.children.for_each(self, state, env, context);
                }
                if let Some(event) = <V::Widget as WidgetEvent>::Event::from_any(self.event) {
                    let result = widget.event(event, &node.children, context.id_path(), state, env);
                    context.process_result(result);
                    self.result = true;
                }
            }
            _ => {}
        }
    }
}
