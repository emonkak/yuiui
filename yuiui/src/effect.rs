use std::any::Any;
use std::fmt;
use std::sync::Arc;

use crate::cancellation_token::CancellationToken;
use crate::command::Command;
use crate::state::State;

pub enum Effect<S: State> {
    Message(S::Message),
    Mutation(Box<dyn FnOnce(&mut S) -> bool + Send>),
    Command(Command<S>, Option<CancellationToken>),
    DownwardEvent(Box<dyn Any + Send>),
    UpwardEvent(Box<dyn Any + Send>),
    LocalEvent(Box<dyn Any + Send>),
    RequestUpdate,
    SubscribeState,
    UnsubscribeState,
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
            Self::DownwardEvent(event) => Effect::DownwardEvent(event),
            Self::UpwardEvent(event) => Effect::UpwardEvent(event),
            Self::LocalEvent(event) => Effect::LocalEvent(event),
            Self::RequestUpdate => Effect::RequestUpdate,
            Self::SubscribeState => Effect::SubscribeState,
            Self::UnsubscribeState => Effect::UnsubscribeState,
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
            Self::DownwardEvent(event) => f.debug_tuple("DownwardEvent").field(event).finish(),
            Self::UpwardEvent(event) => f.debug_tuple("UpwardEvent").field(event).finish(),
            Self::LocalEvent(event) => f.debug_tuple("LocalEvent").field(event).finish(),
            Self::SubscribeState => f.write_str("SubscribeState"),
            Self::UnsubscribeState => f.write_str("UnsubscribeState"),
            Self::RequestUpdate => f.write_str("RequestUpdate"),
        }
    }
}
