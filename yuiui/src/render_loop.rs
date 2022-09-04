use std::collections::VecDeque;
use std::fmt;
use std::time::{Duration, Instant};

use crate::cancellation_token::CancellationToken;
use crate::command::Command;
use crate::context::{EffectContext, RenderContext};
use crate::effect::{Effect, EffectPath};
use crate::element::{Element, ElementSeq};
use crate::id::IdSelection;
use crate::state::State;
use crate::view::View;
use crate::view_node::{CommitMode, ViewNode};

pub struct RenderLoop<El: Element<S, E>, S: State, E> {
    node: ViewNode<El::View, El::Components, S, E>,
    render_context: RenderContext,
    effect_queue: VecDeque<(EffectPath, Effect<S>)>,
    update_selection: IdSelection,
    commit_selection: IdSelection,
    is_mounted: bool,
}

impl<El, S, E> RenderLoop<El, S, E>
where
    El: Element<S, E>,
    S: State,
    E: RenderLoopContext<S>,
{
    pub fn build(element: El, state: &S, env: &E) -> Self {
        let mut context = RenderContext::new();
        let node = element.render(state, env, &mut context);
        Self {
            node,
            render_context: RenderContext::new(),
            effect_queue: VecDeque::new(),
            update_selection: IdSelection::new(),
            commit_selection: IdSelection::new(),
            is_mounted: false,
        }
    }

    pub fn run(&mut self, deadline: &impl Deadline, state: &mut S, env: &E) -> RenderFlow {
        loop {
            while let Some((path, effect)) = self.effect_queue.pop_front() {
                self.apply_effect(path, effect, state, env);
                if deadline.did_timeout() {
                    return self.render_status();
                }
            }

            while let Some((id_path, component_index)) = self.update_selection.pop() {
                self.node.update_subtree(
                    &id_path,
                    component_index,
                    state,
                    env,
                    &mut self.render_context,
                );
                if self.is_mounted {
                    self.commit_selection.select(id_path, component_index);
                }
                if deadline.did_timeout() {
                    return self.render_status();
                }
            }

            if self.is_mounted {
                while let Some((id_path, component_index)) = self.commit_selection.pop() {
                    let mut effect_context = EffectContext::new();
                    self.node.commit_subtree(
                        &id_path,
                        component_index,
                        state,
                        env,
                        &mut effect_context,
                    );
                    self.effect_queue.extend(effect_context.into_effects());
                    if deadline.did_timeout() {
                        return self.render_status();
                    }
                }
            } else {
                let mut effect_context = EffectContext::new();
                self.node
                    .commit(CommitMode::Mount, state, env, &mut effect_context);
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

    pub fn push_effect(&mut self, effect_path: EffectPath, effect: Effect<S>) {
        self.effect_queue.push_back((effect_path, effect));
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

    fn apply_effect(&mut self, effect_path: EffectPath, effect: Effect<S>, state: &mut S, env: &E) {
        match effect {
            Effect::Message(message) => {
                if state.reduce(message) {
                    self.update_selection
                        .select(effect_path.state_id_path, effect_path.state_component_index);
                }
            }
            Effect::Mutation(mutation) => {
                if mutation(state) {
                    self.update_selection
                        .select(effect_path.state_id_path, effect_path.state_component_index);
                }
            }
            Effect::Command(command, cancellation_token) => {
                env.invoke_command(effect_path, command, cancellation_token);
            }
            Effect::DownwardEvent(event) => {
                let mut effect_context = EffectContext::new();
                self.node.downward_event(
                    &event,
                    &effect_path.id_path,
                    state,
                    env,
                    &mut effect_context,
                );
                self.effect_queue.extend(effect_context.into_effects());
            }
            Effect::UpwardEvent(event) => {
                let mut effect_context = EffectContext::new();
                self.node.upward_event(
                    &event,
                    &effect_path.id_path,
                    state,
                    env,
                    &mut effect_context,
                );
                self.effect_queue.extend(effect_context.into_effects());
            }
            Effect::LocalEvent(event) => {
                let mut effect_context = EffectContext::new();
                self.node.local_event(
                    &event,
                    &effect_path.id_path,
                    state,
                    env,
                    &mut effect_context,
                );
                self.effect_queue.extend(effect_context.into_effects());
            }
            Effect::RequestUpdate => {
                self.update_selection
                    .select(effect_path.state_id_path, effect_path.state_component_index);
            }
        }
    }
}

impl<El, S, E> fmt::Debug for RenderLoop<El, S, E>
where
    El: Element<S, E>,
    El::View: fmt::Debug,
    <El::View as View<S, E>>::Widget: fmt::Debug,
    <<El::View as View<S, E>>::Children as ElementSeq<S, E>>::Storage: fmt::Debug,
    El::Components: fmt::Debug,
    S: State,
    S::Message: fmt::Debug,
    E: fmt::Debug,
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
        effect_path: EffectPath,
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
