use std::any::Any;

use crate::component_stack::ComponentStack;
use crate::context::CommitContext;
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

impl<'a, V, CS, S, E> TraversableVisitor<ViewNode<V, CS, S, E>, CommitContext<S>, S, E>
    for UpwardEventVisitor<'a>
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
        context: &mut CommitContext<S>,
    ) -> bool {
        context.set_component_index(CS::LEN);
        match node.state.as_mut().unwrap() {
            ViewNodeState::Prepared(view, widget) | ViewNodeState::Pending(view, _, widget) => {
                let mut captured = false;
                if let Some((head, tail)) = self.id_path.split_first() {
                    self.id_path = tail;
                    captured |= node.children.search(&[*head], self, state, env, context);
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
