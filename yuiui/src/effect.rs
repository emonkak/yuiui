use std::fmt;
use std::sync::Arc;

use crate::cancellation_token::CancellationToken;
use crate::command::Command;
use crate::context::StateScope;
use crate::id::{Depth, IdPathBuf};
use crate::state::State;

pub enum Effect<S: State> {
    Message(S::Message, StateScope),
    Mutation(Box<dyn FnOnce(&mut S) -> bool + Send>, StateScope),
    Command(Command<S>, Option<CancellationToken>),
    RequestUpdate(IdPathBuf, Depth),
}

impl<S: State> Effect<S> {
    pub(crate) fn lift<F, NewState>(self, f: &Arc<F>) -> Effect<NewState>
    where
        F: Fn(&NewState) -> &S + Sync + Send + 'static,
        NewState: State,
    {
        match self {
            Self::Message(message, state_scope) => {
                let f = f.clone();
                Effect::Mutation(
                    Box::new(move |state| {
                        let sub_state: &mut S = unsafe { &mut *(f(state) as *const _ as *mut _) };
                        sub_state.reduce(message)
                    }),
                    state_scope,
                )
            }
            Self::Mutation(mutation, state_scope) => {
                let f = f.clone();
                Effect::Mutation(
                    Box::new(move |state| {
                        let sub_state: &mut S = unsafe { &mut *(f(state) as *const _ as *mut _) };
                        mutation(sub_state)
                    }),
                    state_scope,
                )
            }
            Self::Command(command, cancellation_token) => {
                let f = f.clone();
                let command = command.map(move |effect| effect.lift(&f));
                Effect::Command(command, cancellation_token)
            }
            Self::RequestUpdate(id_path, depth) => Effect::RequestUpdate(id_path, depth),
        }
    }
}

impl<S: State> fmt::Debug for Effect<S>
where
    S::Message: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Message(message, state_scope) => f
                .debug_tuple("Message")
                .field(message)
                .field(state_scope)
                .finish(),
            Self::Mutation(_, state_scope) => f.debug_tuple("Mutation").field(state_scope).finish(),
            Self::Command(command, cancellation_token) => f
                .debug_tuple("Command")
                .field(command)
                .field(cancellation_token)
                .finish(),
            Self::RequestUpdate(id_path, depth) => f
                .debug_tuple("RequestUpdate")
                .field(id_path)
                .field(depth)
                .finish(),
        }
    }
}
