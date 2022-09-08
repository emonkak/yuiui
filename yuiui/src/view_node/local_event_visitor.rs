use std::any::Any;

use crate::component_stack::ComponentStack;
use crate::context::{EffectContext, IdContext};
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

impl<'a, V, CS, S, B> Visitor<ViewNode<V, CS, S, B>, EffectContext<S>, S, B>
    for LocalEventVisitor<'a>
where
    V: View<S, B>,
    CS: ComponentStack<S, B, View = V>,
    S: State,
{
    type Output = bool;

    fn visit(
        &mut self,
        node: &mut ViewNode<V, CS, S, B>,
        state: &S,
        backend: &B,
        context: &mut EffectContext<S>,
    ) -> Self::Output {
        match node.state.as_mut().unwrap() {
            ViewNodeState::Prepared(view, widget) | ViewNodeState::Pending(view, _, widget) => {
                let event = <V as HasEvent>::Event::from_any(self.event)
                    .expect("cast any event to view event");
                let result = view.event(
                    event,
                    widget,
                    &node.children,
                    context.id_path(),
                    state,
                    backend,
                );
                context.process_result(result, CS::LEN);
                true
            }
            ViewNodeState::Uninitialized(_) => false,
        }
    }
}
