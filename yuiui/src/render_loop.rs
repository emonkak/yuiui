use std::any::Any;
use std::collections::{btree_map, BTreeMap, VecDeque};
use std::fmt;
use std::mem;
use std::time::{Duration, Instant};

use crate::cancellation_token::RawToken;
use crate::command::Command;
use crate::context::{EffectContext, RenderContext};
use crate::effect::DestinedEffect;
use crate::element::{Element, ElementSeq};
use crate::event::EventDestination;
use crate::id::{Depth, IdPathBuf, IdTree};
use crate::state::State;
use crate::view::View;
use crate::view_node::{CommitMode, ViewNode};

pub struct RenderLoop<E: Element<S, B>, S: State, B> {
    node: ViewNode<E::View, E::Components, S, B>,
    render_context: RenderContext,
    effect_queue: VecDeque<DestinedEffect<S>>,
    update_selection: BTreeMap<IdPathBuf, Depth>,
    commit_selection: BTreeMap<IdPathBuf, Depth>,
    is_mounted: bool,
}

impl<E, S, B> RenderLoop<E, S, B>
where
    E: Element<S, B>,
    S: State,
    B: RenderLoopContext<S>,
{
    pub fn build(element: E, state: &S, backend: &B) -> Self {
        let mut context = RenderContext::new();
        let node = element.render(&mut context, state, backend);
        Self {
            node,
            render_context: RenderContext::new(),
            effect_queue: VecDeque::new(),
            update_selection: BTreeMap::new(),
            commit_selection: BTreeMap::new(),
            is_mounted: false,
        }
    }

    pub fn run(&mut self, deadline: &impl Deadline, state: &mut S, backend: &B) -> RenderFlow {
        loop {
            while let Some(effect) = self.effect_queue.pop_front() {
                self.run_effect(effect, state, backend);
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
                    let mut effect_context = EffectContext::new();
                    let result =
                        self.node
                            .commit_subtree(&id_tree, &mut effect_context, state, backend);
                    self.effect_queue.extend(result.into_effects());
                    if deadline.did_timeout() {
                        return self.render_status();
                    }
                }
            } else {
                let mut effect_context = EffectContext::new();
                let result =
                    self.node
                        .commit(CommitMode::Mount, &mut effect_context, state, backend);
                self.effect_queue.extend(result.into_effects());
                self.is_mounted = true;
                if deadline.did_timeout() {
                    return self.render_status();
                }
            }

            if self.effect_queue.is_empty() {
                return RenderFlow::Done;
            }
        }
    }

    pub fn dispatch_event(
        &mut self,
        event: Box<dyn Any>,
        destination: EventDestination,
        state: &S,
        backend: &B,
    ) {
        let mut context = EffectContext::new();
        let result = match destination {
            EventDestination::Global => {
                self.node.global_event(&event, &mut context, state, backend)
            }
            EventDestination::Downward(id_path) => {
                self.node
                    .downward_event(&event, &id_path, &mut context, state, backend)
            }
            EventDestination::Upward(id_path) => {
                self.node
                    .upward_event(&event, &id_path, &mut context, state, backend)
            }
            EventDestination::Local(id_path) => {
                self.node
                    .local_event(&event, &id_path, &mut context, state, backend)
            }
        };
        self.effect_queue.extend(result.into_effects());
    }

    pub fn push_effect(&mut self, effect: DestinedEffect<S>) {
        self.effect_queue.push_back(effect);
    }

    fn render_status(&self) -> RenderFlow {
        if self.effect_queue.is_empty()
            && self.update_selection.is_empty()
            && self.commit_selection.is_empty()
            && self.is_mounted
        {
            RenderFlow::Done
        } else {
            RenderFlow::Suspended
        }
    }

    fn run_effect(&mut self, effect: DestinedEffect<S>, state: &mut S, backend: &B) {
        match effect {
            DestinedEffect::Message(message, state_scope) => {
                if state.reduce(message) {
                    let (id_path, depth) = state_scope.normalize();
                    extend_selection(&mut self.update_selection, id_path, depth);
                }
            }
            DestinedEffect::Mutation(mutation, state_scope) => {
                if mutation(state) {
                    let (id_path, depth) = state_scope.normalize();
                    extend_selection(&mut self.update_selection, id_path, depth);
                }
            }
            DestinedEffect::Command(command, cancellation_token, context) => {
                let token = backend.invoke_command(command, context);
                if let Some(cancellation_token) = cancellation_token {
                    cancellation_token.register(token);
                }
            }
            DestinedEffect::RequestUpdate(id_path, depth) => {
                extend_selection(&mut self.update_selection, id_path, depth);
            }
        }
    }
}

impl<E, S, B> fmt::Debug for RenderLoop<E, S, B>
where
    E: Element<S, B>,
    E::View: fmt::Debug,
    <E::View as View<S, B>>::State: fmt::Debug,
    <<E::View as View<S, B>>::Children as ElementSeq<S, B>>::Storage: fmt::Debug,
    E::Components: fmt::Debug,
    S: State,
    S::Message: fmt::Debug,
    B: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RenderLoop")
            .field("node", &self.node)
            .field("render_context", &self.render_context)
            .field("effect_queue", &self.effect_queue)
            .field("update_selection", &self.update_selection)
            .field("commit_selection", &self.commit_selection)
            .field("is_mounted", &self.is_mounted)
            .finish()
    }
}

pub trait RenderLoopContext<S: State> {
    fn invoke_command(&self, command: Command<S>, context: EffectContext) -> RawToken;
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
