use std::mem;

use crate::component_stack::ComponentStack;
use crate::context::RenderContext;
use crate::id::ComponentIndex;
use crate::state::State;
use crate::traversable::{Traversable, TraversableVisitor};
use crate::view::View;

use super::ViewNode;

pub struct UpdateVisitor {
    component_index: ComponentIndex,
}

impl<'a> UpdateVisitor {
    pub fn new(component_index: ComponentIndex) -> Self {
        Self { component_index }
    }
}

impl<V, CS, S, B> TraversableVisitor<ViewNode<V, CS, S, B>, RenderContext, S, B> for UpdateVisitor
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
        context: &mut RenderContext,
    ) -> bool {
        let component_index = mem::replace(&mut self.component_index, 0);
        if component_index < CS::LEN {
            CS::update(
                &mut node.borrow_mut(),
                component_index,
                0,
                state,
                backend,
                context,
            )
        } else {
            node.dirty = true;
            node.children.for_each(self, state, backend, context);
            true
        }
    }
}
