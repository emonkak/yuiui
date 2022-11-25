use std::collections::VecDeque;
use std::time::{Duration, Instant};
use std::{cmp, fmt, mem};

use crate::command::CommandRuntime;
use crate::component_stack::ComponentStack;
use crate::context::{CommitContext, RenderContext};
use crate::effect::Effect;
use crate::element::{Element, ElementSeq};
use crate::event::TransferableEvent;
use crate::id::{IdStack, IdTree, Level};
use crate::state::State;
use crate::view::View;
use crate::view_node::{CommitMode, ViewNode};

pub struct RenderLoop<Element: self::Element<S, M, E>, S, M, E> {
    node: ViewNode<Element::View, Element::Components, S, M, E>,
    id_stack: IdStack,
    message_queue: VecDeque<M>,
    event_queue: VecDeque<TransferableEvent>,
    nodes_to_update: IdTree<Level>,
    nodes_to_commit: IdTree<()>,
    is_mounted: bool,
}

impl<Element, S, M, E> RenderLoop<Element, S, M, E>
where
    Element: self::Element<S, M, E>,
    S: State<Message = M>,
{
    pub fn create(element: Element, state: &S) -> Self {
        let mut id_stack = IdStack::new();
        let mut context = RenderContext {
            id_stack: &mut id_stack,
            state,
            level: Element::Components::LEVEL,
        };
        let node = element.render(&mut context);
        Self {
            node,
            id_stack,
            message_queue: VecDeque::new(),
            event_queue: VecDeque::new(),
            nodes_to_update: IdTree::new(),
            nodes_to_commit: IdTree::new(),
            is_mounted: false,
        }
    }

    pub fn run_until(
        &mut self,
        state: &mut S,
        entry_point: &E,
        command_runtime: &impl CommandRuntime<M>,
        deadline: &Instant,
    ) -> RenderFlow {
        self.run(state, entry_point, command_runtime, deadline)
    }

    pub fn run_forever(
        &mut self,
        state: &mut S,
        entry_point: &E,
        command_runtime: &impl CommandRuntime<M>,
    ) {
        let render_flow = self.run(state, entry_point, command_runtime, &Forever);
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

    fn run(
        &mut self,
        state: &mut S,
        entry_point: &E,
        command_runtime: &impl CommandRuntime<M>,
        deadline: &impl Deadline,
    ) -> RenderFlow {
        loop {
            while let Some(message) = self.message_queue.pop_front() {
                let effect = state.update(message);
                self.process_effect(effect, command_runtime);
                if deadline.did_timeout() {
                    return self.render_flow();
                }
            }

            while let Some(event) = self.event_queue.pop_front() {
                self.process_event(event, state, entry_point);
                if deadline.did_timeout() {
                    return self.render_flow();
                }
            }

            if !self.message_queue.is_empty() {
                continue;
            }

            if !self.nodes_to_update.is_empty() {
                let id_tree = mem::take(&mut self.nodes_to_update);
                let mut context = RenderContext {
                    id_stack: &mut self.id_stack,
                    state,
                    level: Element::Components::LEVEL,
                };
                let changed_nodes = self.node.update_subtree(&id_tree, &mut context);
                if self.is_mounted {
                    for id_path in changed_nodes {
                        self.nodes_to_commit.insert(&id_path, ());
                    }
                }
                if deadline.did_timeout() {
                    return self.render_flow();
                }
            }

            if self.is_mounted {
                if !self.nodes_to_commit.is_empty() {
                    let id_tree = mem::take(&mut self.nodes_to_commit);
                    let mut messages = Vec::new();
                    let mut context = CommitContext {
                        id_stack: &mut self.id_stack,
                        state,
                        messages: &mut messages,
                        entry_point,
                    };
                    self.node.commit_subtree(&id_tree, &mut context);
                    self.message_queue.extend(messages);
                    if deadline.did_timeout() {
                        return self.render_flow();
                    }
                }
            } else {
                let mut messages = Vec::new();
                let mut context = CommitContext {
                    id_stack: &mut self.id_stack,
                    state,
                    messages: &mut messages,
                    entry_point,
                };
                self.node.commit_whole(CommitMode::Mount, &mut context);
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

    fn process_effect(&mut self, effect: Effect<M>, command_runtime: &impl CommandRuntime<M>) {
        let mut current_effect = effect;
        let mut effect_queue = VecDeque::new();
        loop {
            match current_effect {
                Effect::Command(command, cancellation_token) => {
                    command_runtime.spawn_command(command, cancellation_token);
                }
                Effect::Update(subscribers) => {
                    for subscriber in subscribers {
                        self.nodes_to_update.insert_or_update(
                            &subscriber.id_path,
                            subscriber.level,
                            cmp::max,
                        );
                    }
                }
                Effect::ForceUpdate => {
                    self.nodes_to_update.insert_or_update(
                        &[],
                        Element::Components::LEVEL,
                        cmp::max,
                    );
                }
                Effect::Batch(effects) => {
                    effect_queue.extend(effects);
                }
            }
            if let Some(next_effect) = effect_queue.pop_front() {
                current_effect = next_effect;
            } else {
                break;
            }
        }
    }

    fn process_event(&mut self, event: TransferableEvent, state: &mut S, entry_point: &E) {
        let mut messages = Vec::new();
        let mut context = CommitContext {
            id_stack: &mut self.id_stack,
            state,
            messages: &mut messages,
            entry_point,
        };
        match event {
            TransferableEvent::Forward(destination, payload) => {
                self.node
                    .forward_event(&*payload, &destination, &mut context)
            }
            TransferableEvent::Broadcast(destinations, paylaod) => {
                self.node
                    .broadcast_event(&*paylaod, &destinations, &mut context)
            }
        }
        self.message_queue.extend(messages);
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

trait Deadline {
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
