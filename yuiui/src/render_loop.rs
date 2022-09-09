use std::any::Any;
use std::collections::{btree_map, BTreeMap, VecDeque};
use std::fmt;
use std::mem;
use std::time::{Duration, Instant};

use crate::command::CommandRuntime;
use crate::context::{MessageContext, RenderContext, StateScope};
use crate::element::{Element, ElementSeq};
use crate::event::EventDestination;
use crate::id::{Depth, IdPathBuf, IdTree};
use crate::state::State;
use crate::view::View;
use crate::view_node::{CommitMode, ViewNode};

pub struct RenderLoop<E: Element<S, M, B>, S, M, B> {
    node: ViewNode<E::View, E::Components, S, M, B>,
    render_context: RenderContext,
    message_queue: VecDeque<(M, StateScope)>,
    update_selection: BTreeMap<IdPathBuf, Depth>,
    commit_selection: BTreeMap<IdPathBuf, Depth>,
    is_mounted: bool,
}

impl<E, S, M, B> RenderLoop<E, S, M, B>
where
    E: Element<S, M, B>,
    S: State<Message = M>,
{
    pub fn create(element: E, state: &S, backend: &B) -> Self {
        let mut context = RenderContext::new();
        let node = element.render(&mut context, state, backend);
        Self {
            node,
            render_context: RenderContext::new(),
            message_queue: VecDeque::new(),
            update_selection: BTreeMap::new(),
            commit_selection: BTreeMap::new(),
            is_mounted: false,
        }
    }

    pub fn run(
        &mut self,
        deadline: &impl Deadline,
        command_runtime: &impl CommandRuntime<M>,
        state: &mut S,
        backend: &B,
    ) -> RenderFlow {
        loop {
            while let Some((message, state_scope)) = self.message_queue.pop_front() {
                let (dirty, command) = state.update(message);
                if dirty {
                    let (id_path, depth) = state_scope.clone().normalize();
                    extend_selection(&mut self.update_selection, id_path, depth);
                }
                for (command_atom, cancellation_token) in command {
                    command_runtime.spawn_command(
                        command_atom,
                        cancellation_token,
                        state_scope.clone(),
                    );
                }
                if deadline.did_timeout() {
                    return self.render_status();
                }
            }

            if !self.update_selection.is_empty() {
                let id_tree = IdTree::from_iter(mem::take(&mut self.update_selection));
                let changed_nodes =
                    self.node
                        .update_subtree(&id_tree, state, backend, &mut self.render_context);
                if self.is_mounted {
                    for (id_path, depth) in changed_nodes {
                        extend_selection(&mut self.commit_selection, id_path, depth);
                    }
                }
                if deadline.did_timeout() {
                    return self.render_status();
                }
            }

            if self.is_mounted {
                if !self.commit_selection.is_empty() {
                    let id_tree = IdTree::from_iter(mem::take(&mut self.commit_selection));
                    let mut context = MessageContext::new();
                    self.node
                        .commit_subtree(&id_tree, &mut context, state, backend);
                    self.message_queue.extend(context.into_messages());
                    if deadline.did_timeout() {
                        return self.render_status();
                    }
                }
            } else {
                let mut context = MessageContext::new();
                self.node
                    .commit(CommitMode::Mount, &mut context, state, backend);
                self.message_queue.extend(context.into_messages());
                self.is_mounted = true;
                if deadline.did_timeout() {
                    return self.render_status();
                }
            }

            if self.message_queue.is_empty() {
                return RenderFlow::Done;
            }
        }
    }

    pub fn push_message(&mut self, message: M, state_scope: StateScope) {
        self.message_queue.push_back((message, state_scope));
    }

    pub fn dispatch_event(
        &mut self,
        event: Box<dyn Any + Send>,
        destination: EventDestination,
        state: &S,
        backend: &B,
    ) {
        let mut context = MessageContext::new();
        match destination {
            EventDestination::Global => {
                self.node.global_event(&event, &mut context, state, backend);
            }
            EventDestination::Downward(id_path) => {
                self.node
                    .downward_event(&event, &id_path, &mut context, state, backend);
            }
            EventDestination::Upward(id_path) => {
                self.node
                    .upward_event(&event, &id_path, &mut context, state, backend);
            }
            EventDestination::Local(id_path) => {
                self.node
                    .local_event(&event, &id_path, &mut context, state, backend);
            }
        }
        self.message_queue.extend(context.into_messages());
    }

    fn render_status(&self) -> RenderFlow {
        if self.message_queue.is_empty()
            && self.update_selection.is_empty()
            && self.commit_selection.is_empty()
            && self.is_mounted
        {
            RenderFlow::Done
        } else {
            RenderFlow::Suspended
        }
    }
}

impl<E, S, M, B> fmt::Debug for RenderLoop<E, S, M, B>
where
    E: Element<S, M, B>,
    E::View: fmt::Debug,
    <E::View as View<S, M, B>>::State: fmt::Debug,
    <<E::View as View<S, M, B>>::Children as ElementSeq<S, M, B>>::Storage: fmt::Debug,
    E::Components: fmt::Debug,
    M: fmt::Debug,
    B: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RenderLoop")
            .field("node", &self.node)
            .field("render_context", &self.render_context)
            .field("message_queue", &self.message_queue)
            .field("update_selection", &self.update_selection)
            .field("commit_selection", &self.commit_selection)
            .field("is_mounted", &self.is_mounted)
            .finish()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RenderFlow {
    Suspended,
    Done,
}

pub trait Deadline {
    fn did_timeout(&self) -> bool;
}

impl Deadline for Instant {
    fn did_timeout(&self) -> bool {
        self.saturating_duration_since(Instant::now()) <= Duration::from_millis(1)
    }
}

pub struct Forever;

impl Deadline for Forever {
    fn did_timeout(&self) -> bool {
        false
    }
}

fn extend_selection(selection: &mut BTreeMap<IdPathBuf, Depth>, id_path: IdPathBuf, depth: Depth) {
    match selection.entry(id_path) {
        btree_map::Entry::Vacant(entry) => {
            entry.insert(depth);
        }
        btree_map::Entry::Occupied(mut entry) => {
            let current_depth = entry.get_mut();
            *current_depth = (*current_depth).min(depth);
        }
    }
}
