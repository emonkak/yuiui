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

impl<V, CS, S, E> TraversableVisitor<ViewNode<V, CS, S, E>, RenderContext, S, E> for UpdateVisitor
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
        context: &mut RenderContext,
    ) -> bool {
        let component_index = mem::replace(&mut self.component_index, 0);
        if component_index < CS::LEN {
            let scope = node.scope();
            CS::update(scope, component_index, 0, state, env, context)
        } else {
            node.dirty = true;
            node.children.for_each(self, state, env, context);
            true
        }
    }
}
