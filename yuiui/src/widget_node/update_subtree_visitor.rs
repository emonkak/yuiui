use crate::component_node::ComponentStack;
use crate::render::{ComponentIndex, RenderContext, RenderContextSeq, RenderContextVisitor};
use crate::state::State;
use crate::view::View;

use super::{WidgetNode, WidgetState};

pub struct UpdateSubtreeVisitor {
    component_index: Option<ComponentIndex>,
    result: bool,
}

impl UpdateSubtreeVisitor {
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

impl RenderContextVisitor for UpdateSubtreeVisitor {
    fn visit<V, CS, S, E>(
        &mut self,
        node: &mut WidgetNode<V, CS, S, E>,
        state: &S,
        env: &E,
        context: &mut RenderContext,
    ) where
        V: View<S, E>,
        CS: ComponentStack<S, E, View = V>,
        S: State,
    {
        let scope = node.scope();
        let component_index = self.component_index.take().unwrap_or(0);
        if CS::force_update(scope, component_index, 0, state, env, context) {
            self.result = true;
            node.state = match node.state.take().unwrap() {
                WidgetState::Prepared(widget, view) => WidgetState::Dirty(widget, view),
                state @ _ => state,
            }
            .into();
            RenderContextSeq::for_each(&mut node.children, self, state, env, context);
        }
    }
}
