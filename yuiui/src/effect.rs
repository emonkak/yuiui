use std::any::Any;
use std::fmt;
use std::sync::Arc;

use crate::cancellation_token::CancellationToken;
use crate::command::Command;
use crate::id::{ComponentIndex, IdPathBuf};
use crate::state::State;

pub enum Effect<S: State> {
    Message(S::Message),
    Mutation(Box<dyn FnOnce(&mut S) -> bool + Send>),
    Command(Command<S>, Option<CancellationToken>),
    DownwardEvent(Box<dyn Any + Send>),
    UpwardEvent(Box<dyn Any + Send>),
    LocalEvent(Box<dyn Any + Send>),
    RequestUpdate,
}

impl<S: State> Effect<S> {
    pub(crate) fn lift<F, NS>(self, f: &Arc<F>) -> Effect<NS>
    where
        F: Fn(&NS) -> &S + Sync + Send + 'static,
        NS: State,
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
            Self::RequestUpdate => f.write_str("RequestUpdate"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct EffectPath {
    pub id_path: IdPathBuf,
    pub component_index: ComponentIndex,
    pub state_id_path: IdPathBuf,
    pub state_component_index: ComponentIndex,
}

impl EffectPath {
    pub const ROOT: Self = Self::new();

    pub(crate) const fn new() -> Self {
        Self {
            id_path: IdPathBuf::new(),
            component_index: 0,
            state_id_path: IdPathBuf::new(),
            state_component_index: 0,
        }
    }

    pub(crate) fn new_sub_path(&self) -> Self {
        Self {
            id_path: self.id_path.clone(),
            component_index: self.component_index,
            state_id_path: self.id_path.clone(),
            state_component_index: self.component_index,
        }
    }
}
