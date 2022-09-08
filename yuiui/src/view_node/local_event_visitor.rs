use std::any::Any;

use crate::component_stack::ComponentStack;
use crate::context::EffectContext;
use crate::effect::EffectOps;
use crate::event::{Event, HasEvent};
use crate::state::State;
use crate::traversable::Visitor;
use crate::view::View;

use super::{ViewNode, ViewNodeState};

pub struct LocalEventVisitor<'a> {
    event: &'a dyn Any,
}

impl<'a> LocalEventVisitor<'a> {
    pub fn new(event: &'a dyn Any) -> Self {
        Self { event }
    }
}

impl<'a, V, CS, S, B> Visitor<ViewNode<V, CS, S, B>, EffectContext, S, B> for LocalEventVisitor<'a>
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
                let event = <V as HasEvent>::Event::from_any(self.event)
                    .expect("cast any event to view event");
                context.set_depth(CS::LEN);
                view.event(
                    event,
                    view_state,
                    &mut node.children,
                    context,
                    state,
                    backend,
                )
            }
            ViewNodeState::Uninitialized(_) => EffectOps::nop(),
        }
    }
}
