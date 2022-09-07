use std::mem;

use crate::component_stack::ComponentStack;
use crate::context::{CommitContext, IdContext};
use crate::event::Lifecycle;
use crate::id::ComponentIndex;
use crate::state::State;
use crate::traversable::TraversableVisitor;
use crate::view::View;

use super::{CommitMode, ViewNode, ViewNodeSeq, ViewNodeState};

pub struct CommitVisitor {
    mode: CommitMode,
    component_index: ComponentIndex,
}

impl CommitVisitor {
    pub fn new(mode: CommitMode, component_index: ComponentIndex) -> Self {
        Self {
            mode,
            component_index,
        }
    }
}

impl<V, CS, S, E> TraversableVisitor<ViewNode<V, CS, S, E>, CommitContext<S>, S, E>
    for CommitVisitor
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
        let has_changed = node.children.commit(self.mode, state, env, context);
        node.state = match (self.mode, node.state.take().unwrap()) {
            (CommitMode::Mount, ViewNodeState::Uninitialized(view)) => {
                let mut widget = view.build(&node.children, state, env);
                let result = view.lifecycle(
                    Lifecycle::Mounted,
                    &mut widget,
                    &node.children,
                    context.id_path(),
                    state,
                    env,
                );
                context.process_result(result, CS::LEN);
                ViewNodeState::Prepared(view, widget)
            }
            (CommitMode::Mount, ViewNodeState::Prepared(view, mut widget)) => {
                let result = view.lifecycle(
                    Lifecycle::Remounted,
                    &mut widget,
                    &node.children,
                    context.id_path(),
                    state,
                    env,
                );
                context.process_result(result, CS::LEN);
                ViewNodeState::Prepared(view, widget)
            }
            (CommitMode::Mount, ViewNodeState::Pending(view, pending_view, mut widget)) => {
                let result = view.lifecycle(
                    Lifecycle::Remounted,
                    &mut widget,
                    &node.children,
                    context.id_path(),
                    state,
                    env,
                );
                context.process_result(result, CS::LEN);
                let result = pending_view.lifecycle(
                    Lifecycle::Updated(&view),
                    &mut widget,
                    &node.children,
                    context.id_path(),
                    state,
                    env,
                );
                context.process_result(result, CS::LEN);
                ViewNodeState::Prepared(pending_view, widget)
            }
            (CommitMode::Update, ViewNodeState::Uninitialized(_)) => {
                unreachable!()
            }
            (CommitMode::Update, ViewNodeState::Prepared(view, mut widget)) => {
                let result = view.lifecycle(
                    Lifecycle::Updated(&view),
                    &mut widget,
                    &node.children,
                    context.id_path(),
                    state,
                    env,
                );
                context.process_result(result, CS::LEN);
                ViewNodeState::Prepared(view, widget)
            }
            (CommitMode::Update, ViewNodeState::Pending(view, pending_view, mut widget)) => {
                let result = pending_view.lifecycle(
                    Lifecycle::Updated(&view),
                    &mut widget,
                    &node.children,
                    context.id_path(),
                    state,
                    env,
                );
                context.process_result(result, CS::LEN);
                ViewNodeState::Prepared(pending_view, widget)
            }
            (CommitMode::Unmount, ViewNodeState::Uninitialized(_)) => {
                unreachable!()
            }
            (CommitMode::Unmount, ViewNodeState::Prepared(view, mut widget)) => {
                let result = view.lifecycle(
                    Lifecycle::Unmounted,
                    &mut widget,
                    &node.children,
                    context.id_path(),
                    state,
                    env,
                );
                context.process_result(result, CS::LEN);
                ViewNodeState::Prepared(view, widget)
            }
            (CommitMode::Unmount, ViewNodeState::Pending(view, pending_view, mut widget)) => {
                let result = view.lifecycle(
                    Lifecycle::Unmounted,
                    &mut widget,
                    &node.children,
                    context.id_path(),
                    state,
                    env,
                );
                context.process_result(result, CS::LEN);
                ViewNodeState::Pending(view, pending_view, widget)
            }
        }
        .into();
        let component_index = mem::replace(&mut self.component_index, 0);
        if component_index < CS::LEN {
            node.components
                .commit(self.mode, component_index, 0, state, env, context);
        }
        has_changed
    }
}
