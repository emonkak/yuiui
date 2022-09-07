use std::any::Any;

use crate::component_stack::ComponentStack;
use crate::context::{EffectContext, IdContext};
use crate::event::{Event, HasEvent};
use crate::id::IdPath;
use crate::state::State;
use crate::traversable::{Traversable, TraversableVisitor};
use crate::view::View;

use super::{ViewNode, ViewNodeState};

pub struct UpwardEventVisitor<'a> {
    event: &'a dyn Any,
    id_path: &'a IdPath,
}

impl<'a> UpwardEventVisitor<'a> {
    pub fn new(event: &'a dyn Any, id_path: &'a IdPath) -> Self {
        Self { event, id_path }
    }
}

impl<'a, V, CS, S, B> TraversableVisitor<ViewNode<V, CS, S, B>, EffectContext<S>, S, B>
    for UpwardEventVisitor<'a>
where
    V: View<S, B>,
    CS: ComponentStack<S, B, View = V>,
    S: State,
{
    fn visit(
        &mut self,
        node: &mut ViewNode<V, CS, S, B>,
        state: &S,
        backend: &B,
        context: &mut EffectContext<S>,
    ) -> bool {
        match node.state.as_mut().unwrap() {
            ViewNodeState::Prepared(view, widget) | ViewNodeState::Pending(view, _, widget) => {
                let mut captured = false;
                if let Some((head, tail)) = self.id_path.split_first() {
                    self.id_path = tail;
                    captured |= node
                        .children
                        .search(&[*head], self, state, backend, context);
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
