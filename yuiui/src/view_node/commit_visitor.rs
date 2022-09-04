use std::mem;

use crate::component_node::ComponentStack;
use crate::context::EffectContext;
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

impl<V, CS, S, E> TraversableVisitor<ViewNode<V, CS, S, E>, EffectContext<S>, S, E>
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
        context: &mut EffectContext<S>,
    ) {
        let component_index = mem::replace(&mut self.component_index, 0);
        if component_index < CS::LEN {
            node.components
                .commit(self.mode, component_index, state, env, context);
        }
        node.children.commit(self.mode, state, env, context);
        node.state = match (self.mode, node.state.take().unwrap()) {
            (CommitMode::Mount, ViewNodeState::Uninitialized(view)) => {
                let mut widget = view.build(&node.children, state, env);
                let result = view.lifecycle(
                    Lifecycle::Mounted,
                    &mut widget,
                    &node.children,
                    context.effect_path(),
                    state,
                    env,
                );
                context.process_result(result);
                ViewNodeState::Prepared(view, widget)
            }
            (CommitMode::Mount, ViewNodeState::Prepared(view, mut widget)) => {
                let result = view.lifecycle(
                    Lifecycle::Remounted,
                    &mut widget,
                    &node.children,
                    context.effect_path(),
                    state,
                    env,
                );
                context.process_result(result);
                ViewNodeState::Prepared(view, widget)
            }
            (CommitMode::Mount, ViewNodeState::Pending(view, pending_view, mut widget)) => {
                let result = view.lifecycle(
                    Lifecycle::Remounted,
                    &mut widget,
                    &node.children,
                    context.effect_path(),
                    state,
                    env,
                );
                context.process_result(result);
                let result = pending_view.lifecycle(
                    Lifecycle::Updated(&view),
                    &mut widget,
                    &node.children,
                    context.effect_path(),
                    state,
                    env,
                );
                context.process_result(result);
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
                    context.effect_path(),
                    state,
                    env,
                );
                context.process_result(result);
                ViewNodeState::Prepared(view, widget)
            }
            (CommitMode::Update, ViewNodeState::Pending(view, pending_view, mut widget)) => {
                let result = pending_view.lifecycle(
                    Lifecycle::Updated(&view),
                    &mut widget,
                    &node.children,
                    context.effect_path(),
                    state,
                    env,
                );
                context.process_result(result);
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
                    context.effect_path(),
                    state,
                    env,
                );
                context.process_result(result);
                ViewNodeState::Prepared(view, widget)
            }
            (CommitMode::Unmount, ViewNodeState::Pending(view, pending_view, mut widget)) => {
                let result = view.lifecycle(
                    Lifecycle::Unmounted,
                    &mut widget,
                    &node.children,
                    context.effect_path(),
                    state,
                    env,
                );
                context.process_result(result);
                ViewNodeState::Pending(view, pending_view, widget)
            }
        }
        .into();
    }
}
