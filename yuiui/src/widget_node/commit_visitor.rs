use std::mem;

use crate::component_node::ComponentStack;
use crate::context::EffectContext;
use crate::event::Lifecycle;
use crate::id::ComponentIndex;
use crate::state::State;
use crate::traversable::TraversableVisitor;
use crate::view::View;

use super::{CommitMode, WidgetNode, WidgetNodeSeq, WidgetState};

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
        let component_index = mem::replace(&mut self.component_index, 0);
        if component_index < CS::LEN {
            node.components
                .commit(self.mode, component_index, state, env, context);
        }
        node.children.commit(self.mode, state, env, context);
        node.state = match (self.mode, node.state.take().unwrap()) {
            (CommitMode::Mount, WidgetState::Uninitialized(view)) => {
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
                WidgetState::Prepared(widget, view)
            }
            (CommitMode::Mount, WidgetState::Prepared(mut widget, view)) => {
                let result = view.lifecycle(
                    Lifecycle::Remounted,
                    &mut widget,
                    &node.children,
                    context.effect_path(),
                    state,
                    env,
                );
                context.process_result(result);
                WidgetState::Prepared(widget, view)
            }
            (CommitMode::Mount, WidgetState::Pending(mut widget, view, pending_view)) => {
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
                WidgetState::Prepared(widget, pending_view)
            }
            (CommitMode::Update, WidgetState::Uninitialized(_)) => {
                unreachable!()
            }
            (CommitMode::Update, WidgetState::Prepared(mut widget, view)) => {
                let result = view.lifecycle(
                    Lifecycle::Updated(&view),
                    &mut widget,
                    &node.children,
                    context.effect_path(),
                    state,
                    env,
                );
                context.process_result(result);
                WidgetState::Prepared(widget, view)
            }
            (CommitMode::Update, WidgetState::Pending(mut widget, view, pending_view)) => {
                let result = pending_view.lifecycle(
                    Lifecycle::Updated(&view),
                    &mut widget,
                    &node.children,
                    context.effect_path(),
                    state,
                    env,
                );
                context.process_result(result);
                WidgetState::Prepared(widget, pending_view)
            }
            (CommitMode::Unmount, WidgetState::Uninitialized(_)) => {
                unreachable!()
            }
            (CommitMode::Unmount, WidgetState::Prepared(mut widget, view)) => {
                let result = view.lifecycle(
                    Lifecycle::Unmounted,
                    &mut widget,
                    &node.children,
                    context.effect_path(),
                    state,
                    env,
                );
                context.process_result(result);
                WidgetState::Prepared(widget, view)
            }
            (CommitMode::Unmount, WidgetState::Pending(mut widget, view, pending_view)) => {
                let result = view.lifecycle(
                    Lifecycle::Unmounted,
                    &mut widget,
                    &node.children,
                    context.effect_path(),
                    state,
                    env,
                );
                context.process_result(result);
                WidgetState::Pending(widget, view, pending_view)
            }
        }
        .into();
    }
}
