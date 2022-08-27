use crate::component_node::ComponentStack;
use crate::context::RenderContext;
use crate::id::ComponentIndex;
use crate::sequence::{TraversableSeq, TraversableSeqVisitor};
use crate::state::State;
use crate::view::View;

use super::{WidgetNode, WidgetState};

pub struct UpdateSubtreeVisitor {
    component_index: Option<ComponentIndex>,
    result: bool,
}

impl<'a> UpdateSubtreeVisitor {
    pub fn new(component_index: Option<ComponentIndex>) -> Self {
        Self {
            component_index,
            result: false,
        }
    }

    pub fn result(&self) -> bool {
        self.result
    }
}

impl<V, CS, S, E> TraversableSeqVisitor<WidgetNode<V, CS, S, E>, RenderContext, S, E>
    for UpdateSubtreeVisitor
where
    V: View<S, E>,
    CS: ComponentStack<S, E, View = V>,
    S: State,
{
    fn visit(
        &mut self,
        node: &mut WidgetNode<V, CS, S, E>,
        state: &S,
        env: &E,
        context: &mut RenderContext,
    ) {
        if CS::LEN > 0 {
            let scope = node.scope();
            let component_index = self.component_index.take().unwrap_or(0);
            self.result |= CS::force_update(scope, component_index, 0, state, env, context)
        } else {
            self.result = true;
            node.state = match node.state.take().unwrap() {
                WidgetState::Prepared(widget, view) => WidgetState::Dirty(widget, view),
                state @ _ => state,
            }
            .into();
            node.dirty = true;
            node.children.for_each(self, state, env, context);
        }
    }
}