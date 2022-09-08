use std::mem;

use crate::component_stack::ComponentStack;
use crate::context::RenderContext;
use crate::id::ComponentIndex;
use crate::state::State;
use crate::traversable::{Traversable, Visitor};
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

impl<V, CS, S, B> Visitor<ViewNode<V, CS, S, B>, RenderContext, S, B> for UpdateVisitor
where
    V: View<S, B>,
    CS: ComponentStack<S, B, View = V>,
    S: State,
{
    type Output = bool;

    fn visit(
        &mut self,
        node: &mut ViewNode<V, CS, S, B>,
        context: &mut RenderContext,
        state: &S,
        backend: &B,
    ) -> Self::Output {
        let component_index = mem::replace(&mut self.component_index, 0);
        if component_index < CS::LEN {
            CS::update(
                &mut node.borrow_mut(),
                component_index,
                0,
                context,
                state,
                backend,
            )
        } else {
            node.dirty = true;
            node.children.for_each(self, context, state, backend);
            true
        }
    }
}
