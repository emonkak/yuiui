use std::any::Any;

use crate::component_stack::ComponentStack;
use crate::context::EffectContext;
use crate::event::{Event, EventResult, HasEvent};
use crate::state::State;
use crate::traversable::{Monoid, Traversable, Visitor};
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

impl<'a, V, CS, S, B> Visitor<ViewNode<V, CS, S, B>, EffectContext, S, B>
    for DownwardEventVisitor<'a>
where
    V: View<S, B>,
    CS: ComponentStack<S, B, View = V>,
    S: State,
{
    type Output = EventResult<S>;

    fn visit(
        &mut self,
        node: &mut ViewNode<V, CS, S, B>,
        context: &mut EffectContext,
        state: &S,
        backend: &B,
    ) -> Self::Output {
        match node.state.as_mut().unwrap() {
            ViewNodeState::Prepared(view, widget) | ViewNodeState::Pending(view, _, widget) => {
                let mut result = EventResult::nop();
                if node.event_mask.contains(&self.event.type_id()) {
                    result = result.combine(node.children.for_each(self, context, state, backend));
                }
                if let Some(event) = <V as HasEvent>::Event::from_any(self.event) {
                    context.begin_effect(CS::LEN);
                    result = result.combine(view.event(
                        event,
                        widget,
                        &node.children,
                        context,
                        state,
                        backend,
                    ));
                }
                result
            }
            _ => EventResult::nop(),
        }
    }
}
