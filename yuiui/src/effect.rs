use std::fmt;
use std::sync::Arc;

use crate::cancellation_token::CancellationToken;
use crate::command::Command;
use crate::state::State;

pub enum Effect<S: State> {
    Message(S::Message),
    Mutation(Box<dyn FnOnce(&mut S) -> bool + Send>),
    Command(Command<S>, Option<CancellationToken>),
    RequestUpdate,
}

impl<S: State> Effect<S> {
    pub(crate) fn lift<F, NewState>(self, f: &Arc<F>) -> Effect<NewState>
    where
        F: Fn(&NewState) -> &S + Sync + Send + 'static,
        NewState: State,
    {
        match self {
            Self::Message(message) => {
                let f = f.clone();
                Effect::Mutation(Box::new(move |state| {
                    let sub_state: &mut S = unsafe { &mut *(f(state) as *const _ as *mut _) };
                    sub_state.reduce(message)
                }))
            }
            Self::Mutation(mutation) => {
                let f = f.clone();
                Effect::Mutation(Box::new(move |state| {
                    let sub_state: &mut S = unsafe { &mut *(f(state) as *const _ as *mut _) };
                    mutation(sub_state)
                }))
            }
            Self::Command(command, cancellation_token) => {
                let f = f.clone();
                let command = command.map(move |effect| effect.lift(&f));
                Effect::Command(command, cancellation_token)
            }
            Self::RequestUpdate => Effect::RequestUpdate,
        }
    }
}

impl<S: State> fmt::Debug for Effect<S>
where
    S::Message: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Message(message) => f.debug_tuple("Message").field(message).finish(),
            Self::Mutation(_) => f.debug_struct("Mutation").finish_non_exhaustive(),
            Self::Command(command, cancellation_token) => f
                .debug_tuple("Command")
                .field(command)
                .field(cancellation_token)
                .finish(),
            Self::RequestUpdate => f.write_str("RequestUpdate"),
        }
    }
}
