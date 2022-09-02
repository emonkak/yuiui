use std::collections::VecDeque;
use std::fmt;
use std::time::{Duration, Instant};

use crate::cancellation_token::CancellationToken;
use crate::command::Command;
use crate::context::{EffectContext, RenderContext};
use crate::effect::{Effect, EffectPath};
use crate::element::{Element, ElementSeq};
use crate::id::{ComponentIndex, IdPath};
use crate::state::State;
use crate::view::View;
use crate::widget_node::{CommitMode, WidgetNode, WidgetNodeSeq};

pub struct RenderLoop<El: Element<S, E>, S: State, E> {
    node: WidgetNode<El::View, El::Components, S, E>,
    state: S,
    env: E,
    render_context: RenderContext,
    effect_queue: VecDeque<(EffectPath, Effect<S>)>,
    update_queue: VecDeque<(IdPath, ComponentIndex)>,
    commit_queue: VecDeque<(IdPath, ComponentIndex)>,
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
            update_queue: VecDeque::new(),
            commit_queue: VecDeque::new(),
            is_mounted: false,
        }
    }

    pub fn run(&mut self, deadline: &Instant) {
        if deadline_did_timeout(&deadline) {
            self.request_render();
            return;
        }

        loop {
            while let Some((path, effect)) = self.effect_queue.pop_front() {
                self.apply_effect(path, effect);
                if deadline_did_timeout(&deadline) {
                    self.request_render();
                    return;
                }
            }

            while let Some((id_path, component_index)) = self.update_queue.pop_front() {
                self.node.update_subtree(
                    &id_path,
                    component_index,
                    &self.state,
                    &self.env,
                    &mut self.render_context,
                );
                if self.is_mounted {
                    self.commit_queue.push_back((id_path, component_index));
                }
                if deadline_did_timeout(&deadline) {
                    self.request_render();
                    return;
                }
            }

            if self.is_mounted {
                while let Some((id_path, component_index)) = self.commit_queue.pop_front() {
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
                        self.request_render();
                        return;
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
                    self.request_render();
                    return;
                }
            }

            if self.effect_queue.is_empty() {
                return;
            }
        }
    }

    pub fn push_effect(&mut self, effect_path: EffectPath, effect: Effect<S>) {
        self.effect_queue.push_back((effect_path, effect));
    }

    fn apply_effect(&mut self, effect_path: EffectPath, effect: Effect<S>) {
        match effect {
            Effect::Message(message) => {
                if self.state.reduce(message) {
                    self.update_queue
                        .push_back((effect_path.state_id_path, effect_path.state_component_index));
                }
            }
            Effect::Mutation(mutation) => {
                if mutation(&mut self.state) {
                    self.update_queue
                        .push_back((effect_path.state_id_path, effect_path.state_component_index));
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
                self.node
                    .internal_event(&event, &effect_path.id_path, &self.state, &self.env, &mut effect_context);
                self.effect_queue.extend(effect_context.into_effects());
            }
            Effect::RequestUpdate => {
                self.update_queue
                    .push_front((effect_path.id_path, effect_path.component_index));
            }
        }
    }

    fn request_render(&self) {
        let is_finished = self.effect_queue.is_empty()
            && self.update_queue.is_empty()
            && self.commit_queue.is_empty()
            && self.is_mounted;
        if !is_finished {
            self.env.request_render();
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
            .field("update_queue", &self.update_queue)
            .field("commit_queue", &self.commit_queue)
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
