use std::mem;

use crate::component_stack::ComponentStack;
use crate::context::MessageContext;
use crate::event::Lifecycle;
use crate::id::Depth;
use crate::state::Store;
use crate::traversable::Visitor;
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

impl<V, CS, S, M, B> Visitor<ViewNode<V, CS, S, M, B>, S, B> for CommitVisitor
where
    V: View<S, M, B>,
    CS: ComponentStack<S, M, B, View = V>,
{
    type Context = MessageContext<M>;

    type Output = bool;

    fn visit(
        &mut self,
        node: &mut ViewNode<V, CS, S, M, B>,
        context: &mut MessageContext<M>,
        store: &Store<S>,
        backend: &B,
    ) -> Self::Output {
        let mut result = false;
        context.set_depth(CS::LEN);
        node.state = match (self.mode, node.state.take().unwrap()) {
            (CommitMode::Mount, ViewNodeState::Uninitialized(view)) => {
                let mut view_state = view.build(&node.children, store, backend);
                node.children.commit(self.mode, context, store, backend);
                view.lifecycle(
                    Lifecycle::Mount,
                    &mut view_state,
                    &node.children,
                    context,
                    store,
                    backend,
                );
                result = true;
                ViewNodeState::Prepared(view, view_state)
            }
            (CommitMode::Mount, ViewNodeState::Prepared(view, mut view_state)) => {
                node.children.commit(self.mode, context, store, backend);
                view.lifecycle(
                    Lifecycle::Mount,
                    &mut view_state,
                    &node.children,
                    context,
                    store,
                    backend,
                );
                result = true;
                ViewNodeState::Prepared(view, view_state)
            }
            (CommitMode::Mount, ViewNodeState::Pending(view, pending_view, mut view_state)) => {
                node.children.commit(self.mode, context, store, backend);
                view.lifecycle(
                    Lifecycle::Mount,
                    &mut view_state,
                    &node.children,
                    context,
                    store,
                    backend,
                );
                pending_view.lifecycle(
                    Lifecycle::Update(&view),
                    &mut view_state,
                    &node.children,
                    context,
                    store,
                    backend,
                );
                result = true;
                ViewNodeState::Prepared(pending_view, view_state)
            }
            (CommitMode::Update, ViewNodeState::Uninitialized(_)) => {
                unreachable!()
            }
            (CommitMode::Update, ViewNodeState::Prepared(view, view_state)) => {
                result |= node.children.commit(self.mode, context, store, backend);
                ViewNodeState::Prepared(view, view_state)
            }
            (CommitMode::Update, ViewNodeState::Pending(view, pending_view, mut view_state)) => {
                node.children.commit(self.mode, context, store, backend);
                pending_view.lifecycle(
                    Lifecycle::Update(&view),
                    &mut view_state,
                    &node.children,
                    context,
                    store,
                    backend,
                );
                result = true;
                ViewNodeState::Prepared(pending_view, view_state)
            }
            (CommitMode::Unmount, ViewNodeState::Uninitialized(_)) => {
                unreachable!()
            }
            (CommitMode::Unmount, ViewNodeState::Prepared(view, mut view_state)) => {
                view.lifecycle(
                    Lifecycle::Unmount,
                    &mut view_state,
                    &node.children,
                    context,
                    store,
                    backend,
                );
                node.children.commit(self.mode, context, store, backend);
                result = true;
                ViewNodeState::Prepared(view, view_state)
            }
            (CommitMode::Unmount, ViewNodeState::Pending(view, pending_view, mut view_state)) => {
                view.lifecycle(
                    Lifecycle::Unmount,
                    &mut view_state,
                    &node.children,
                    context,
                    store,
                    backend,
                );
                node.children.commit(self.mode, context, store, backend);
                result = true;
                ViewNodeState::Pending(view, pending_view, view_state)
            }
        }
        .into();

        let depth = mem::replace(&mut self.depth, 0);
        if depth < CS::LEN {
            result |= node
                .components
                .commit(self.mode, depth, 0, context, store, backend);
        }

        result
    }
}
