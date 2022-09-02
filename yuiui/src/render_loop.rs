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
use crate::widget_node::{CommitMode, WidgetNode, WidgetNodeSeq};

pub struct RenderLoop<El: Element<S, E>, S: State, E> {
    node: WidgetNode<El::View, El::Components, S, E>,
    state: S,
    env: E,
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
    pub fn build(element: El, state: S, env: E) -> Self {
        let mut context = RenderContext::new();
        let node = element.render(&state, &env, &mut context);
        Self {
            node,
            state,
            env,
            render_context: RenderContext::new(),
            effect_queue: VecDeque::new(),
            update_selection: IdSelection::new(),
            commit_selection: IdSelection::new(),
            is_mounted: false,
        }
    }

    pub fn run(&mut self, deadline: &Instant) -> RenderStatus {
        if deadline_did_timeout(&deadline) {
            return self.schedule_render();
        }

        loop {
            while let Some((path, effect)) = self.effect_queue.pop_front() {
                self.apply_effect(path, effect);
                if deadline_did_timeout(&deadline) {
                    return self.schedule_render();
                }
            }

            while let Some((id_path, component_index)) = self.update_selection.pop() {
                self.node.update_subtree(
                    &id_path,
                    component_index,
                    &self.state,
                    &self.env,
                    &mut self.render_context,
                );
                if self.is_mounted {
                    self.commit_selection.select(id_path, component_index);
                }
                if deadline_did_timeout(&deadline) {
                    return self.schedule_render();
                }
            }

            if self.is_mounted {
                while let Some((id_path, component_index)) = self.commit_selection.pop() {
                    let mut effect_context = EffectContext::new();
                    self.node.commit_subtree(
                        &id_path,
                        component_index,
                        &self.state,
                        &self.env,
                        &mut effect_context,
                    );
                    self.effect_queue.extend(effect_context.into_effects());
                    if deadline_did_timeout(&deadline) {
                        return self.schedule_render();
                    }
                }
            } else {
                let mut effect_context = EffectContext::new();
                self.node.commit(
                    CommitMode::Mount,
                    &self.state,
                    &self.env,
                    &mut effect_context,
                );
                self.effect_queue.extend(effect_context.into_effects());
                self.is_mounted = true;
                if deadline_did_timeout(&deadline) {
                    return self.schedule_render();
                }
            }

            if self.effect_queue.is_empty() {
                return RenderStatus::Done;
            }
        }
    }

    pub fn push_effect(&mut self, effect_path: EffectPath, effect: Effect<S>) {
        self.effect_queue.push_back((effect_path, effect));
    }

    fn schedule_render(&self) -> RenderStatus {
        let is_finished = self.effect_queue.is_empty()
            && self.update_selection.is_empty()
            && self.commit_selection.is_empty()
            && self.is_mounted;
        if !is_finished {
            self.env.request_render();
            RenderStatus::Suspended
        } else {
            RenderStatus::Done
        }
    }

    fn apply_effect(&mut self, effect_path: EffectPath, effect: Effect<S>) {
        match effect {
            Effect::Message(message) => {
                if self.state.reduce(message) {
                    self.update_selection
                        .select(effect_path.state_id_path, effect_path.state_component_index);
                }
            }
            Effect::Mutation(mutation) => {
                if mutation(&mut self.state) {
                    self.update_selection
                        .select(effect_path.state_id_path, effect_path.state_component_index);
                }
            }
            Effect::Command(command, cancellation_token) => {
                self.env
                    .invoke_command(effect_path, command, cancellation_token);
            }
            Effect::Event(event) => {
                let mut effect_context = EffectContext::new();
                self.node
                    .event(&event, &self.state, &self.env, &mut effect_context);
                self.effect_queue.extend(effect_context.into_effects());
            }
            Effect::InternalEvent(event) => {
                let mut effect_context = EffectContext::new();
                self.node.internal_event(
                    &event,
                    &effect_path.id_path,
                    &self.state,
                    &self.env,
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
    <<El::View as View<S, E>>::Children as ElementSeq<S, E>>::Store: fmt::Debug,
    El::Components: fmt::Debug,
    S: State + fmt::Debug,
    S::Message: fmt::Debug,
    E: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RenderLoop")
            .field("node", &self.node)
            .field("state", &self.state)
            .field("env", &self.env)
            .field("render_context", &self.render_context)
            .field("effect_queue", &self.effect_queue)
            .field("update_selection", &self.update_selection)
            .field("commit_selection", &self.commit_selection)
            .field("is_mounted", &self.is_mounted)
            .finish()
    }
}

pub trait RenderLoopContext<S: State> {
    fn request_render(&self);

    fn invoke_command(
        &self,
        effect_path: EffectPath,
        command: Command<S>,
        cancellation_token: Option<CancellationToken>,
    );
}

fn deadline_did_timeout(deadline: &Instant) -> bool {
    deadline.saturating_duration_since(Instant::now()) <= Duration::from_millis(1)
}

#[derive(Debug)]
pub enum RenderStatus {
    Suspended,
    Done,
}
