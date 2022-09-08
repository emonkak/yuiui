use std::mem;

use crate::component_stack::ComponentStack;
use crate::context::EffectContext;
use crate::event::{EventResult, Lifecycle};
use crate::id::Depth;
use crate::state::State;
use crate::traversable::{Monoid, Visitor};
use crate::view::View;

use super::{CommitMode, ViewNode, ViewNodeSeq, ViewNodeState};

pub struct CommitVisitor {
    mode: CommitMode,
    depth: Depth,
}

impl CommitVisitor {
    pub fn new(mode: CommitMode, depth: Depth) -> Self {
        Self { mode, depth }
    }
}

impl<V, CS, S, B> Visitor<ViewNode<V, CS, S, B>, EffectContext, S, B> for CommitVisitor
where
    V: View<S, B>,
    CS: ComponentStack<S, B, View = V>,
    S: State,
{
    type Output = EventResult<S>;

    fn visit(
        &mut self,
        node: &mut ViewNode<V, CS, S, B>,
        context: &mut EffectContext,
        state: &S,
        backend: &B,
    ) -> Self::Output {
        let mut result = node.children.commit(self.mode, context, state, backend);
        context.begin_depth(CS::LEN);
        let node_state = match (self.mode, node.state.take().unwrap()) {
            (CommitMode::Mount, ViewNodeState::Uninitialized(view)) => {
                let mut widget = view.build(&node.children, state, backend);
                result = result.combine(view.lifecycle(
                    Lifecycle::Mounted,
                    &mut widget,
                    &node.children,
                    context,
                    state,
                    backend,
                ));
                ViewNodeState::Prepared(view, widget)
            }
            (CommitMode::Mount, ViewNodeState::Prepared(view, mut widget)) => {
                result = result.combine(view.lifecycle(
                    Lifecycle::Mounted,
                    &mut widget,
                    &node.children,
                    context,
                    state,
                    backend,
                ));
                ViewNodeState::Prepared(view, widget)
            }
            (CommitMode::Mount, ViewNodeState::Pending(view, pending_view, mut widget)) => {
                result = result
                    .combine(view.lifecycle(
                        Lifecycle::Mounted,
                        &mut widget,
                        &node.children,
                        context,
                        state,
                        backend,
                    ))
                    .combine(pending_view.lifecycle(
                        Lifecycle::Updated(&view),
                        &mut widget,
                        &node.children,
                        context,
                        state,
                        backend,
                    ));
                ViewNodeState::Prepared(pending_view, widget)
            }
            (CommitMode::Update, ViewNodeState::Uninitialized(_)) => {
                unreachable!()
            }
            (CommitMode::Update, ViewNodeState::Prepared(view, mut widget)) => {
                result = result.combine(view.lifecycle(
                    Lifecycle::Updated(&view),
                    &mut widget,
                    &node.children,
                    context,
                    state,
                    backend,
                ));
                ViewNodeState::Prepared(view, widget)
            }
            (CommitMode::Update, ViewNodeState::Pending(view, pending_view, mut widget)) => {
                result = result.combine(pending_view.lifecycle(
                    Lifecycle::Updated(&view),
                    &mut widget,
                    &node.children,
                    context,
                    state,
                    backend,
                ));
                ViewNodeState::Prepared(pending_view, widget)
            }
            (CommitMode::Unmount, ViewNodeState::Uninitialized(_)) => {
                unreachable!()
            }
            (CommitMode::Unmount, ViewNodeState::Prepared(view, mut widget)) => {
                result = result.combine(view.lifecycle(
                    Lifecycle::Unmounted,
                    &mut widget,
                    &node.children,
                    context,
                    state,
                    backend,
                ));
                ViewNodeState::Prepared(view, widget)
            }
            (CommitMode::Unmount, ViewNodeState::Pending(view, pending_view, mut widget)) => {
                result = result.combine(view.lifecycle(
                    Lifecycle::Unmounted,
                    &mut widget,
                    &node.children,
                    context,
                    state,
                    backend,
                ));
                ViewNodeState::Pending(view, pending_view, widget)
            }
        };
        node.state = Some(node_state);
        let depth = mem::replace(&mut self.depth, 0);
        if depth < CS::LEN {
            result = result.combine(
                node.components
                    .commit(self.mode, depth, 0, context, state, backend),
            );
        }
        result
    }
}
