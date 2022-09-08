use std::any::Any;

use crate::component_stack::ComponentStack;
use crate::context::{EffectContext, IdContext};
use crate::event::{Event, HasEvent};
use crate::state::State;
use crate::traversable::{Traversable, Visitor};
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

impl<'a, V, CS, S, B> Visitor<ViewNode<V, CS, S, B>, EffectContext<S>, S, B>
    for DownwardEventVisitor<'a>
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
                let mut captured = false;
                if node.event_mask.contains(&self.event.type_id()) {
                    captured |= node.children.for_each(self, state, backend, context);
                }
                if let Some(event) = <V as HasEvent>::Event::from_any(self.event) {
                    let result = view.event(
                        event,
                        widget,
                        &node.children,
                        context.id_path(),
                        state,
                        backend,
                    );
                    context.process_result(result, CS::LEN);
                    captured = true;
                }
                captured
            }
            _ => false,
        }
    }
}
