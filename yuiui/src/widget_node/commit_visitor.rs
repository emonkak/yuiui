use crate::component_node::ComponentStack;
use crate::effect::{EffectContext, EffectContextVisitor};
use crate::state::State;
use crate::view::View;
use crate::widget::{Widget, WidgetLifeCycle};

use super::{CommitMode, WidgetNode, WidgetNodeSeq, WidgetState};

pub struct CommitVisitor {
    mode: CommitMode,
}

impl CommitVisitor {
    pub fn new(mode: CommitMode) -> Self {
        Self { mode }
    }
}

impl EffectContextVisitor for CommitVisitor {
    fn visit<V, CS, S, E>(
        &mut self,
        node: &mut WidgetNode<V, CS, S, E>,
        state: &S,
        env: &E,
        context: &mut EffectContext<S>,
    ) where
        V: View<S, E>,
        CS: ComponentStack<S, E, View = V>,
        S: State,
    {
        context.begin_components();
        node.components.commit(self.mode, state, env, context);
        context.end_components();
        node.children.commit(self.mode, state, env, context);
        node.state = match node.state.take().unwrap() {
            WidgetState::Uninitialized(view) => {
                let mut widget = view.build(&node.children, state, env);
                let result = widget.lifecycle(
                    WidgetLifeCycle::Mounted,
                    &node.children,
                    context.id_path(),
                    state,
                    env,
                );
                context.process_result(result);
                WidgetState::Prepared(widget, view)
            }
            WidgetState::Prepared(mut widget, view) => {
                match self.mode {
                    CommitMode::Mount => {
                        let result = widget.lifecycle(
                            WidgetLifeCycle::Mounted,
                            &node.children,
                            context.id_path(),
                            state,
                            env,
                        );
                        context.process_result(result);
                    }
                    CommitMode::Unmount => {
                        let result = widget.lifecycle(
                            WidgetLifeCycle::Unmounted,
                            &node.children,
                            context.id_path(),
                            state,
                            env,
                        );
                        context.process_result(result);
                    }
                    CommitMode::Update => {}
                }
                WidgetState::Prepared(widget, view)
            }
            WidgetState::Dirty(mut widget, view) => {
                if view.rebuild(&node.children, &mut widget, state, env) {
                    let result = widget.lifecycle(
                        WidgetLifeCycle::Updated,
                        &node.children,
                        context.id_path(),
                        state,
                        env,
                    );
                    context.process_result(result);
                }
                WidgetState::Prepared(widget, view)
            }
        }
        .into();
    }
}
