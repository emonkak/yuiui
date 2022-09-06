use std::any::Any;

use crate::component_stack::ComponentStack;
use crate::context::EffectContext;
use crate::event::{Event, HasEvent};
use crate::state::State;
use crate::traversable::{Traversable, TraversableVisitor};
use crate::view::View;

use super::{ViewNode, ViewNodeState};

pub struct DownwardEventVisitor<'a> {
    event: &'a dyn Any,
}

impl<'a> DownwardEventVisitor<'a> {
    pub fn new(event: &'a dyn Any) -> Self {
        Self { event }
    }
}

impl<'a, V, CS, S, E> TraversableVisitor<ViewNode<V, CS, S, E>, EffectContext<S>, S, E>
    for DownwardEventVisitor<'a>
where
    V: View<S, E>,
    CS: ComponentStack<S, E, View = V>,
    S: State,
{
    fn visit(
        &mut self,
        node: &mut ViewNode<V, CS, S, E>,
        state: &S,
        env: &E,
        context: &mut EffectContext<S>,
    ) -> bool {
        context.set_component_index(CS::LEN);
        match node.state.as_mut().unwrap() {
            ViewNodeState::Prepared(view, widget) | ViewNodeState::Pending(view, _, widget) => {
                let mut captured = false;
                if node.event_mask.contains(&self.event.type_id()) {
                    captured |= node.children.for_each(self, state, env, context);
                }
                if let Some(event) = <V as HasEvent>::Event::from_any(self.event) {
                    let result = view.event(
                        event,
                        widget,
                        &node.children,
                        context.effect_path(),
                        state,
                        env,
                    );
                    context.process_result(result);
                    captured = true;
                }
                captured
            }
            _ => false,
        }
    }
}
