use std::any::Any;
use std::collections::{BTreeMap, VecDeque};
use std::fmt;
use std::mem;
use std::time::{Duration, Instant};

use crate::command::ExecutionContext;
use crate::context::{MessageContext, RenderContext};
use crate::element::{Element, ElementSeq};
use crate::event::EventDestination;
use crate::id::{Depth, IdPath, IdPathBuf, IdTree};
use crate::state::{State, Store};
use crate::view::View;
use crate::view_node::{CommitMode, ViewNode};

pub struct RenderLoop<E: Element<S, M, R>, S, M, R> {
    node: ViewNode<E::View, E::Components, S, M, R>,
    render_context: RenderContext,
    message_queue: VecDeque<M>,
    event_queue: VecDeque<(Box<dyn Any + Send + 'static>, EventDestination)>,
    nodes_to_update: BTreeMap<IdPathBuf, Depth>,
    nodes_to_commit: BTreeMap<IdPathBuf, Depth>,
    is_mounted: bool,
}

impl<E, S, M, R> RenderLoop<E, S, M, R>
where
    E: Element<S, M, R>,
    S: State<Message = M>,
{
    pub fn create(element: E, store: &Store<S>) -> Self {
        let mut context = RenderContext::new();
        let node = element.render(&mut context, store);
        Self {
            node,
            render_context: RenderContext::new(),
            message_queue: VecDeque::new(),
            event_queue: VecDeque::new(),
            nodes_to_update: BTreeMap::new(),
            nodes_to_commit: BTreeMap::new(),
            is_mounted: false,
        }
    }

    pub fn run(
        &mut self,
        deadline: &impl Deadline,
        execution_context: &impl ExecutionContext<M>,
        store: &mut Store<S>,
        renderer: &mut R,
    ) -> RenderFlow {
        loop {
            while let Some(message) = self.message_queue.pop_front() {
                let (dirty, effect) = store.update(message);
                if dirty {
                    if !self.nodes_to_update.contains_key(&[] as &IdPath) {
                        self.nodes_to_update.insert(IdPathBuf::new(), 0);
                    }
                }
                for (id_path, depth) in effect.subscribers {
                    if let Some(current_depth) = self.nodes_to_update.get_mut(&id_path) {
                        *current_depth = (*current_depth).min(depth);
                    } else {
                        self.nodes_to_update.insert(id_path, depth);
                    }
                }
                for (command, cancellation_token) in effect.commands {
                    execution_context.spawn_command(command, cancellation_token);
                }
                if deadline.did_timeout() {
                    return self.render_flow();
                }
            }

            while let Some((event, destination)) = self.event_queue.pop_front() {
                self.dispatch_event(event, destination, store, renderer);
                if deadline.did_timeout() {
                    return self.render_flow();
                }
            }

            if !self.message_queue.is_empty() {
                continue;
            }

            if !self.nodes_to_update.is_empty() {
                let id_tree = IdTree::from_iter(mem::take(&mut self.nodes_to_update));
                let changed_nodes =
                    self.node
                        .update_subtree(&id_tree, store, renderer, &mut self.render_context);
                if self.is_mounted {
                    for (id_path, depth) in changed_nodes {
                        if let Some(current_depth) = self.nodes_to_commit.get_mut(&id_path) {
                            *current_depth = (*current_depth).min(depth);
                        } else {
                            self.nodes_to_commit.insert(id_path, depth);
                        }
                    }
                }
                if deadline.did_timeout() {
                    return self.render_flow();
                }
            }

            if self.is_mounted {
                if !self.nodes_to_commit.is_empty() {
                    let id_tree = IdTree::from_iter(mem::take(&mut self.nodes_to_commit));
                    let mut context = MessageContext::new();
                    self.node
                        .commit_subtree(&id_tree, &mut context, store, renderer);
                    self.message_queue.extend(context.into_messages());
                    if deadline.did_timeout() {
                        return self.render_flow();
                    }
                }
            } else {
                let mut context = MessageContext::new();
                self.node
                    .commit_within(CommitMode::Mount, 0, &mut context, store, renderer);
                self.message_queue.extend(context.into_messages());
                self.is_mounted = true;
                if deadline.did_timeout() {
                    return self.render_flow();
                }
            }

            if self.message_queue.is_empty() {
                return RenderFlow::Done;
            }
        }
    }

    pub fn run_forever(
        &mut self,
        execution_context: &impl ExecutionContext<M>,
        store: &mut Store<S>,
        renderer: &mut R,
    ) {
        let render_flow = self.run(&Forever, execution_context, store, renderer);
        assert_eq!(render_flow, RenderFlow::Done);
    }

    pub fn push_message(&mut self, message: M) {
        self.message_queue.push_back(message);
    }

    pub fn push_event(&mut self, event: Box<dyn Any + Send>, destination: EventDestination) {
        self.event_queue.push_back((event, destination));
    }

    pub fn node(&self) -> &ViewNode<E::View, E::Components, S, M, R> {
        &self.node
    }

    fn dispatch_event(
        &mut self,
        event: Box<dyn Any + Send>,
        destination: EventDestination,
        store: &Store<S>,
        renderer: &mut R,
    ) {
        let mut context = MessageContext::new();
        match destination {
            EventDestination::Global => {
                self.node
                    .global_event(&*event, &mut context, store, renderer);
            }
            EventDestination::Downward(id_path) => {
                self.node
                    .downward_event(&*event, &id_path, &mut context, store, renderer);
            }
            EventDestination::Upward(id_path) => {
                self.node
                    .upward_event(&*event, &id_path, &mut context, store, renderer);
            }
            EventDestination::Local(id_path) => {
                self.node
                    .local_event(&*event, &id_path, &mut context, store, renderer);
            }
        }
        self.message_queue.extend(context.into_messages());
    }

    fn render_flow(&self) -> RenderFlow {
        if self.message_queue.is_empty()
            && self.event_queue.is_empty()
            && self.nodes_to_update.is_empty()
            && self.nodes_to_commit.is_empty()
            && self.is_mounted
        {
            RenderFlow::Done
        } else {
            RenderFlow::Suspend
        }
    }
}

impl<E, S, M, R> fmt::Debug for RenderLoop<E, S, M, R>
where
    E: Element<S, M, R>,
    E::View: fmt::Debug,
    <E::View as View<S, M, R>>::State: fmt::Debug,
    <<E::View as View<S, M, R>>::Children as ElementSeq<S, M, R>>::Storage: fmt::Debug,
    E::Components: fmt::Debug,
    M: fmt::Debug,
    R: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RenderLoop")
            .field("node", &self.node)
            .field("render_context", &self.render_context)
            .field("message_queue", &self.message_queue)
            .field("event_queue", &self.event_queue)
            .field("nodes_to_update", &self.nodes_to_update)
            .field("nodes_to_commit", &self.nodes_to_commit)
            .field("is_mounted", &self.is_mounted)
            .finish()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RenderFlow {
    Suspend,
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

struct Forever;

impl Deadline for Forever {
    fn did_timeout(&self) -> bool {
        false
    }
}
