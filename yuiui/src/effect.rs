use std::sync::Arc;

use crate::command::{Command, CommandId};
use crate::event::EventResult;
use crate::id::{ComponentIndex, Id, IdPath};
use crate::state::State;

pub enum Effect<S: State> {
    Message(S::Message),
    Mutation(Box<dyn FnOnce(&mut S) -> bool + Send>),
    Command(Command<S>),
    IdentifiedCommand(CommandId, Command<S>),
    CancelCommand(CommandId),
    CancelAllCommands,
}

impl<S: State> Effect<S> {
    pub(crate) fn lift<F, PS>(self, f: Arc<F>) -> Effect<PS>
    where
        S: 'static,
        F: Fn(&PS) -> &S + Sync + Send + 'static,
        PS: State,
    {
        match self {
            Self::Message(message) => Effect::Mutation(Box::new(move |state| {
                let sub_state: &mut S = unsafe { &mut *(f(state) as *const _ as *mut _) };
                sub_state.reduce(message)
            })),
            Self::Mutation(mutation) => Effect::Mutation(Box::new(move |state| {
                let sub_state: &mut S = unsafe { &mut *(f(state) as *const _ as *mut _) };
                mutation(sub_state)
            })),
            Self::Command(command) => {
                let command = command.map(move |effect| effect.lift(f.clone()));
                Effect::Command(command)
            }
            Self::IdentifiedCommand(command_id, command) => {
                let command = command.map(move |effect| effect.lift(f.clone()));
                Effect::IdentifiedCommand(command_id, command)
            }
            Self::CancelCommand(id) => Effect::CancelCommand(id),
            Self::CancelAllCommands => Effect::CancelAllCommands,
        }
    }
}

pub type PendingEffects<S> = Vec<(IdPath, Option<ComponentIndex>, Effect<S>)>;

pub struct EffectContext<S: State> {
    id_path: IdPath,
    component_index: Option<ComponentIndex>,
    state_id_path: IdPath,
    state_component_index: Option<ComponentIndex>,
    pending_effects: PendingEffects<S>,
}

impl<S: State> EffectContext<S> {
    pub fn new() -> Self {
        Self {
            id_path: IdPath::new(),
            component_index: None,
            state_id_path: IdPath::new(),
            state_component_index: None,
            pending_effects: Vec::new(),
        }
    }

    pub fn new_sub_context<SS: State>(&self) -> EffectContext<SS> {
        EffectContext {
            id_path: self.id_path.clone(),
            component_index: self.component_index,
            state_id_path: self.id_path.clone(),
            state_component_index: self.component_index,
            pending_effects: Vec::new(),
        }
    }

    pub fn id_path(&self) -> &IdPath {
        &self.id_path
    }

    pub fn merge<F, SS>(&mut self, sub_context: EffectContext<SS>, f: Arc<F>)
    where
        S: 'static,
        F: Fn(&S) -> &SS + Sync + Send + 'static,
        SS: State + 'static,
    {
        assert!(sub_context.id_path.starts_with(&self.id_path));

        let pending_effects =
            sub_context
                .pending_effects
                .into_iter()
                .map(|(id_path, component_index, message)| {
                    (id_path, component_index, message.lift(f.clone()))
                });
        self.pending_effects.extend(pending_effects);
    }

    pub fn begin_widget(&mut self, id: Id) {
        self.id_path.push(id);
    }

    pub fn end_widget(&mut self) -> Id {
        self.id_path.pop()
    }

    pub fn begin_components(&mut self) {
        self.component_index = Some(0);
    }

    pub fn next_component(&mut self) {
        *self.component_index.as_mut().unwrap() += 1;
    }

    pub fn end_components(&mut self) {
        self.component_index = None;
    }

    pub fn process_result(&mut self, result: EventResult<S>) {
        for effect in result.into_effects() {
            self.pending_effects.push((
                self.state_id_path.clone(),
                self.state_component_index,
                effect,
            ));
        }
    }

    pub fn into_effects(self) -> PendingEffects<S> {
        self.pending_effects
    }
}
