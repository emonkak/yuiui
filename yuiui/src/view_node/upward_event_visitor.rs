use std::any::Any;

use crate::component_stack::ComponentStack;
use crate::context::EffectContext;
use crate::effect::EffectOps;
use crate::event::{Event, HasEvent};
use crate::id::IdPath;
use crate::traversable::{Monoid, Traversable, Visitor};
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

impl<'a, V, CS, S, M, B> Visitor<ViewNode<V, CS, S, M, B>, EffectContext, S, B>
    for UpwardEventVisitor<'a>
where
    V: View<S, M, B>,
    CS: ComponentStack<S, M, B, View = V>,
{
    type Output = EffectOps<M>;

    fn visit(
        &mut self,
        node: &mut ViewNode<V, CS, S, M, B>,
        context: &mut EffectContext,
        state: &S,
        backend: &B,
    ) -> Self::Output {
        match node.state.as_mut().unwrap() {
            ViewNodeState::Prepared(view, view_state)
            | ViewNodeState::Pending(view, _, view_state) => {
                let mut result = EffectOps::nop();
                if let Some((head, tail)) = self.id_path.split_first() {
                    self.id_path = tail;
                    if let Some(child_captured) =
                        node.children
                            .search(&[*head], self, context, state, backend)
                    {
                        result = result.combine(child_captured);
                    }
                }
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
                result
            }
            _ => EffectOps::nop(),
        }
    }
}
