use std::any::Any;

use crate::component_stack::ComponentStack;
use crate::context::EffectContext;
use crate::effect::EffectOps;
use crate::event::{Event, HasEvent};
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
    type Output = EffectOps<S>;

    fn visit(
        &mut self,
        node: &mut ViewNode<V, CS, S, B>,
        context: &mut EffectContext,
        state: &S,
        backend: &B,
    ) -> Self::Output {
        match node.state.as_mut().unwrap() {
            ViewNodeState::Prepared(view, view_state)
            | ViewNodeState::Pending(view, _, view_state) => {
                let mut result = EffectOps::nop();
                if let Some(event) = <V as HasEvent>::Event::from_any(self.event) {
                    context.set_depth(CS::LEN);
                    result = result.combine(view.event(
                        event,
                        view_state,
                        &node.children,
                        context,
                        state,
                        backend,
                    ));
                }
                if node.event_mask.contains(&self.event.type_id()) {
                    result = result.combine(node.children.for_each(self, context, state, backend));
                }
                result
            }
            _ => EffectOps::nop(),
        }
    }
}
