use std::collections::VecDeque;
use std::time::{Duration, Instant};
use std::{cmp, fmt, mem};

use crate::command::CommandRuntime;
use crate::component_stack::ComponentStack;
use crate::element::{Element, ElementSeq};
use crate::event::TransferableEvent;
use crate::id::{Depth, IdStack, IdTree};
use crate::store::{State, Store};
use crate::view::View;
use crate::view_node::{CommitMode, ViewNode};

pub struct RenderLoop<Element: self::Element<S, M, E>, S, M, E> {
    node: ViewNode<Element::View, Element::Components, S, M, E>,
    id_stack: IdStack,
    message_queue: VecDeque<M>,
    event_queue: VecDeque<TransferableEvent>,
    nodes_to_update: IdTree<Depth>,
    nodes_to_commit: IdTree<Depth>,
    is_mounted: bool,
}

impl<Element, S, M, E> RenderLoop<Element, S, M, E>
where
    Element: self::Element<S, M, E>,
    S: State<Message = M>,
{
    pub fn create(element: Element, store: &Store<S>) -> Self {
        let mut context = IdStack::new();
        let node = element.render(&mut context, store.state());
        Self {
            node,
            id_stack: context,
            message_queue: VecDeque::new(),
            event_queue: VecDeque::new(),
            nodes_to_update: IdTree::new(),
            nodes_to_commit: IdTree::new(),
            is_mounted: false,
        }
    }

    pub fn run(
        &mut self,
        store: &mut Store<S>,
        entry_point: &E,
        command_runtime: &impl CommandRuntime<M>,
        deadline: &impl Deadline,
    ) -> RenderFlow {
        loop {
            while let Some(message) = self.message_queue.pop_front() {
                let (dirty, effect) = store.update(message);
                if dirty {
                    self.nodes_to_update
                        .insert(&[], <Element::Components as ComponentStack<S, M, E>>::DEPTH);
                }
                for (id_path, depth) in effect.subscribers {
                    self.nodes_to_update
                        .insert_or_update(&id_path, depth, cmp::min);
                }
                for (command, cancellation_token) in effect.commands {
                    command_runtime.spawn_command(command, cancellation_token);
                }
                if deadline.did_timeout() {
                    return self.render_flow();
                }
            }

            while let Some(event) = self.event_queue.pop_front() {
                let messages = match event {
                    TransferableEvent::Forward(destination, payload) => self.node.forward_event(
                        &*payload,
                        &destination,
                        &mut self.id_stack,
                        store,
                        entry_point,
                    ),
                    TransferableEvent::Broadcast(destinations, paylaod) => {
                        self.node.broadcast_event(
                            &*paylaod,
                            &destinations,
                            &mut self.id_stack,
                            store,
                            entry_point,
                        )
                    }
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
                let changed_nodes = self
                    .node
                    .update_subtree(&id_tree, store, &mut self.id_stack);
                if self.is_mounted {
                    for (id_path, depth) in changed_nodes {
                        self.nodes_to_commit
                            .insert_or_update(&id_path, depth, cmp::max);
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
                            .commit_subtree(&id_tree, &mut self.id_stack, store, entry_point);
                    self.message_queue.extend(messages);
                    if deadline.did_timeout() {
                        return self.render_flow();
                    }
                }
            } else {
                let mut messages = Vec::new();
                self.node.commit_from(
                    CommitMode::Mount,
                    0,
                    &mut self.id_stack,
                    store,
                    &mut messages,
                    entry_point,
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
        store: &mut Store<S>,
        entry_point: &E,
        command_runtime: &impl CommandRuntime<M>,
    ) {
        let render_flow = self.run(store, entry_point, command_runtime, &Forever);
        assert_eq!(render_flow, RenderFlow::Done);
    }

    pub fn push_message(&mut self, message: M) {
        self.message_queue.push_back(message);
    }

    pub fn push_event(&mut self, event: TransferableEvent) {
        self.event_queue.push_back(event);
    }

    pub fn node(&self) -> &ViewNode<Element::View, Element::Components, S, M, E> {
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

impl<Element, S, M, E> fmt::Debug for RenderLoop<Element, S, M, E>
where
    Element: self::Element<S, M, E>,
    Element::View: fmt::Debug,
    <Element::View as View<S, M, E>>::State: fmt::Debug,
    <<Element::View as View<S, M, E>>::Children as ElementSeq<S, M, E>>::Storage: fmt::Debug,
    Element::Components: fmt::Debug,
    M: fmt::Debug,
    E: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RenderLoop")
            .field("node", &self.node)
            .field("id_stack", &self.id_stack)
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
