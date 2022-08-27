use std::sync::Arc;

use crate::command::{Command, CommandId};
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
