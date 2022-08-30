use crate::component_node::ComponentStack;
use crate::context::{EffectContext, IdContext};
use crate::event::Lifecycle;
use crate::id::ComponentIndex;
use crate::state::State;
use crate::traversable::TraversableVisitor;
use crate::view::View;

use super::{CommitMode, WidgetNode, WidgetNodeSeq, WidgetState};

pub struct CommitVisitor {
    mode: CommitMode,
    component_index: Option<ComponentIndex>,
}

impl CommitVisitor {
    pub fn new(mode: CommitMode, component_index: Option<ComponentIndex>) -> Self {
        Self {
            mode,
            component_index,
        }
    }
}

impl<V, CS, S, E> TraversableVisitor<WidgetNode<V, CS, S, E>, EffectContext<S>, S, E>
    for CommitVisitor
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
        context: &mut EffectContext<S>,
    ) {
        context.begin_components();
        let component_index = self.component_index.take().unwrap_or(0);
        node.components
            .commit(self.mode, component_index, state, env, context);
        context.end_components();
        node.children.commit(self.mode, state, env, context);
        node.state = match node.state.take().unwrap() {
            WidgetState::Uninitialized(view) => {
                let mut widget = view.build(&node.children, state, env);
                let result = view.lifecycle(
                    Lifecycle::Mounted,
                    &mut widget,
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
                        let result = view.lifecycle(
                            Lifecycle::Mounted,
                            &mut widget,
                            &node.children,
                            context.id_path(),
                            state,
                            env,
                        );
                        context.process_result(result);
                    }
                    CommitMode::Unmount => {
                        let result = view.lifecycle(
                            Lifecycle::Unmounted,
                            &mut widget,
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
                    let result = view.lifecycle(
                        Lifecycle::Updated(&view),
                        &mut widget,
                        &node.children,
                        context.id_path(),
                        state,
                        env,
                    );
                    context.process_result(result);
                }
                WidgetState::Prepared(widget, view)
            }
            WidgetState::Pending(mut widget, view, pending_view) => {
                if view.rebuild(&node.children, &mut widget, state, env) {
                    let result = pending_view.lifecycle(
                        Lifecycle::Updated(&view),
                        &mut widget,
                        &node.children,
                        context.id_path(),
                        state,
                        env,
                    );
                    context.process_result(result);
                }
                WidgetState::Prepared(widget, pending_view)
            }
        }
        .into();
    }
}
