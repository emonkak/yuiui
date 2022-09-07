use std::any::Any;
use std::collections::{btree_map, BTreeMap, VecDeque};
use std::fmt;
use std::mem;
use std::time::{Duration, Instant};

use crate::cancellation_token::CancellationToken;
use crate::command::Command;
use crate::context::{EffectContext, RenderContext};
use crate::effect::Effect;
use crate::element::{Element, ElementSeq};
use crate::event::EventDestination;
use crate::id::{ComponentIndex, IdPathBuf, IdTree};
use crate::state::State;
use crate::view::View;
use crate::view_node::{CommitMode, ViewNode};

pub struct RenderLoop<E: Element<S, B>, S: State, B> {
    node: ViewNode<E::View, E::Components, S, B>,
    render_context: RenderContext,
    effect_queue: VecDeque<(IdPathBuf, ComponentIndex, Effect<S>)>,
    update_selection: BTreeMap<IdPathBuf, ComponentIndex>,
    commit_selection: BTreeMap<IdPathBuf, ComponentIndex>,
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
        let node = element.render(state, backend, &mut context);
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
            while let Some((id_path, component_index, effect)) = self.effect_queue.pop_front() {
                self.run_effect(id_path, component_index, effect, state, backend);
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
                    for (id_path, component_index) in changed_nodes {
                        extend_selection(&mut self.commit_selection, id_path, component_index);
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
                    self.node
                        .commit_subtree(&id_tree, state, backend, &mut effect_context);
                    self.effect_queue.extend(effect_context.into_effects());
                    if deadline.did_timeout() {
                        return self.render_status();
                    }
                }
            } else {
                let mut effect_context = EffectContext::new();
                self.node
                    .commit(CommitMode::Mount, state, backend, &mut effect_context);
                self.effect_queue.extend(effect_context.into_effects());
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
        match destination {
            EventDestination::Global => {
                self.node.global_event(&event, state, backend, &mut context);
            }
            EventDestination::Downward(id_path) => {
                self.node
                    .downward_event(&event, &id_path, state, backend, &mut context);
            }
            EventDestination::Upward(id_path) => {
                self.node
                    .upward_event(&event, &id_path, state, backend, &mut context);
            }
            EventDestination::Local(id_path) => {
                self.node
                    .local_event(&event, &id_path, state, backend, &mut context);
            }
        }
        self.effect_queue.extend(context.into_effects());
    }

    pub fn push_effect(
        &mut self,
        id_path: IdPathBuf,
        component_index: ComponentIndex,
        effect: Effect<S>,
    ) {
        self.effect_queue
            .push_back((id_path, component_index, effect));
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

    fn run_effect(
        &mut self,
        id_path: IdPathBuf,
        component_index: ComponentIndex,
        effect: Effect<S>,
        state: &mut S,
        backend: &B,
    ) {
        match effect {
            Effect::Message(message) => {
                if state.reduce(message) {
                    extend_selection(&mut self.update_selection, id_path, component_index);
                }
            }
            Effect::Mutation(mutation) => {
                if mutation(state) {
                    extend_selection(&mut self.update_selection, id_path, component_index);
                }
            }
            Effect::Command(command, cancellation_token) => {
                backend.invoke_command(id_path, component_index, command, cancellation_token);
            }
            Effect::RequestUpdate => {
                extend_selection(&mut self.update_selection, id_path, component_index);
            }
        }
    }
}

impl<E, S, B> fmt::Debug for RenderLoop<E, S, B>
where
    E: Element<S, B>,
    E::View: fmt::Debug,
    <E::View as View<S, B>>::Widget: fmt::Debug,
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
    fn invoke_command(
        &self,
        id_path: IdPathBuf,
        component_index: ComponentIndex,
        command: Command<S>,
        cancellation_token: Option<CancellationToken>,
    );
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

fn extend_selection(
    selection: &mut BTreeMap<IdPathBuf, ComponentIndex>,
    id_path: IdPathBuf,
    component_index: ComponentIndex,
) {
    match selection.entry(id_path) {
        btree_map::Entry::Vacant(entry) => {
            entry.insert(component_index);
        }
        btree_map::Entry::Occupied(mut entry) => {
            let current_component_index = entry.get_mut();
            *current_component_index = (*current_component_index).min(component_index);
        }
    }
}
