use std::collections::VecDeque;
use std::time::{Duration, Instant};
use std::{cmp, fmt, mem};

use crate::command::CommandContext;
use crate::element::{Element, ElementSeq};
use crate::event::Event;
use crate::id::{Depth, IdContext, IdTree};
use crate::store::{State, Store};
use crate::view::View;
use crate::view_node::{CommitMode, ViewNode};

pub struct RenderLoop<E: Element<S, M, R>, S, M, R> {
    node: ViewNode<E::View, E::Components, S, M, R>,
    id_context: IdContext,
    message_queue: VecDeque<M>,
    event_queue: VecDeque<Event>,
    nodes_to_update: IdTree<Depth>,
    nodes_to_commit: IdTree<Depth>,
    is_mounted: bool,
}

impl<E, S, M, R> RenderLoop<E, S, M, R>
where
    E: Element<S, M, R>,
    S: State<Message = M>,
{
    pub fn create(element: E, store: &Store<S>) -> Self {
        let mut context = IdContext::new();
        let node = element.render(&mut context, store);
        Self {
            node,
            id_context: context,
            message_queue: VecDeque::new(),
            event_queue: VecDeque::new(),
            nodes_to_update: IdTree::new(),
            nodes_to_commit: IdTree::new(),
            is_mounted: false,
        }
    }

    pub fn run(
        &mut self,
        deadline: &impl Deadline,
        command_context: &impl CommandContext<M>,
        store: &mut Store<S>,
        renderer: &mut R,
    ) -> RenderFlow {
        loop {
            while let Some(message) = self.message_queue.pop_front() {
                let (dirty, effect) = store.update(message);
                if dirty {
                    self.nodes_to_update.insert(&[], 0);
                }
                for (id_path, depth) in effect.subscribers {
                    self.nodes_to_update
                        .insert_or_update(&id_path, depth, cmp::min);
                }
                for (command, cancellation_token) in effect.commands {
                    command_context.spawn_command(command, cancellation_token);
                }
                if deadline.did_timeout() {
                    return self.render_flow();
                }
            }

            while let Some(event) = self.event_queue.pop_front() {
                let messages = match event {
                    Event::Forward(destination, payload) => self.node.forward_event(
                        &destination,
                        &*payload,
                        &mut self.id_context,
                        store,
                        renderer,
                    ),
                    Event::Broadcast(destinations, paylaod) => self.node.broadcast_event(
                        &destinations,
                        &*paylaod,
                        &mut self.id_context,
                        store,
                        renderer,
                    ),
                };
                self.message_queue.extend(messages);
                if deadline.did_timeout() {
                    return self.render_flow();
                }
            }

            if !self.message_queue.is_empty() {
                continue;
            }

            if !self.nodes_to_update.is_empty() {
                let id_tree = mem::take(&mut self.nodes_to_update);
                let changed_nodes =
                    self.node
                        .update_subtree(&id_tree, store, renderer, &mut self.id_context);
                if self.is_mounted {
                    for (id_path, depth) in changed_nodes {
                        self.nodes_to_commit
                            .insert_or_update(&id_path, depth, cmp::min);
                    }
                }
                if deadline.did_timeout() {
                    return self.render_flow();
                }
            }

            if self.is_mounted {
                if !self.nodes_to_commit.is_empty() {
                    let id_tree = mem::take(&mut self.nodes_to_commit);
                    let messages =
                        self.node
                            .commit_subtree(&id_tree, &mut self.id_context, store, renderer);
                    self.message_queue.extend(messages);
                    if deadline.did_timeout() {
                        return self.render_flow();
                    }
                }
            } else {
                let mut messages = Vec::new();
                self.node.commit_within(
                    CommitMode::Mount,
                    0,
                    &mut self.id_context,
                    store,
                    &mut messages,
                    renderer,
                );
                self.message_queue.extend(messages);
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
        command_context: &impl CommandContext<M>,
        store: &mut Store<S>,
        renderer: &mut R,
    ) {
        let render_flow = self.run(&Forever, command_context, store, renderer);
        assert_eq!(render_flow, RenderFlow::Done);
    }

    pub fn push_message(&mut self, message: M) {
        self.message_queue.push_back(message);
    }

    pub fn push_event(&mut self, event: Event) {
        self.event_queue.push_back(event);
    }

    pub fn node(&self) -> &ViewNode<E::View, E::Components, S, M, R> {
        &self.node
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
            .field("id_context", &self.id_context)
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
