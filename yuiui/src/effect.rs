use std::fmt;
use std::sync::Arc;

use crate::cancellation_token::CancellationToken;
use crate::command::Command;
use crate::context::{EffectContext, StateScope};
use crate::id::{Depth, IdPathBuf};
use crate::state::State;
use crate::traversable::Monoid;

pub enum Effect<S: State> {
    Message(S::Message),
    Mutation(Box<dyn FnOnce(&mut S) -> bool + Send>),
    Command(Command<S>, Option<CancellationToken>),
    RequestUpdate,
}

impl<S: State> Effect<S> {
    pub fn destine(self, context: &EffectContext) -> DestinedEffect<S> {
        match self {
            Self::Message(message) => {
                DestinedEffect::Message(message, context.state_scope().clone())
            }
            Self::Mutation(mutation) => {
                DestinedEffect::Mutation(mutation, context.state_scope().clone())
            }
            Self::Command(command, cancellation_token) => {
                DestinedEffect::Command(command, cancellation_token, context.clone())
            }
            Self::RequestUpdate => {
                DestinedEffect::RequestUpdate(context.id_path().to_vec(), context.depth())
            }
        }
    }

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
            Self::Mutation(_) => f.debug_tuple("Mutation").finish(),
            Self::Command(command, cancellation_token) => f
                .debug_tuple("Command")
                .field(command)
                .field(cancellation_token)
                .finish(),
            Self::RequestUpdate => f.debug_tuple("RequestUpdate").finish(),
        }
    }
}

pub enum DestinedEffect<S: State> {
    Message(S::Message, StateScope),
    Mutation(Box<dyn FnOnce(&mut S) -> bool + Send>, StateScope),
    Command(Command<S>, Option<CancellationToken>, EffectContext),
    RequestUpdate(IdPathBuf, Depth),
}

impl<S: State> DestinedEffect<S> {
    pub(crate) fn lift<F, NewState>(self, f: &Arc<F>) -> DestinedEffect<NewState>
    where
        F: Fn(&NewState) -> &S + Sync + Send + 'static,
        NewState: State,
    {
        match self {
            Self::Message(message, state_scope) => {
                let f = f.clone();
                DestinedEffect::Mutation(
                    Box::new(move |state| {
                        let sub_state: &mut S = unsafe { &mut *(f(state) as *const _ as *mut _) };
                        sub_state.reduce(message)
                    }),
                    state_scope,
                )
            }
            Self::Mutation(mutation, state_scope) => {
                let f = f.clone();
                DestinedEffect::Mutation(
                    Box::new(move |state| {
                        let sub_state: &mut S = unsafe { &mut *(f(state) as *const _ as *mut _) };
                        mutation(sub_state)
                    }),
                    state_scope,
                )
            }
            Self::Command(command, cancellation_token, context) => {
                let f = f.clone();
                let command = command.map(move |effect| effect.lift(&f));
                DestinedEffect::Command(command, cancellation_token, context)
            }
            Self::RequestUpdate(id_path, depth) => DestinedEffect::RequestUpdate(id_path, depth),
        }
    }
}

impl<S: State> fmt::Debug for DestinedEffect<S>
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
            Self::Command(command, cancellation_token, context) => f
                .debug_tuple("Command")
                .field(command)
                .field(cancellation_token)
                .field(context)
                .finish(),
            Self::RequestUpdate(id_path, depth) => f
                .debug_tuple("RequestUpdate")
                .field(id_path)
                .field(depth)
                .finish(),
        }
    }
}

#[must_use]
pub struct EffectOps<S: State> {
    effects: Vec<DestinedEffect<S>>,
}

impl<S: State> EffectOps<S> {
    pub fn nop() -> Self {
        EffectOps {
            effects: Vec::new(),
        }
    }

    pub fn into_effects(self) -> Vec<DestinedEffect<S>> {
        self.effects
    }

    pub(crate) fn lift<F, NewState>(self, f: &Arc<F>) -> EffectOps<NewState>
    where
        F: Fn(&NewState) -> &S + Sync + Send + 'static,
        NewState: State,
    {
        let effects = self
            .effects
            .into_iter()
            .map(|effect| effect.lift(f))
            .collect();
        EffectOps { effects }
    }
}

impl<S: State> Default for EffectOps<S> {
    fn default() -> Self {
        EffectOps::nop()
    }
}

impl<S: State> Monoid for EffectOps<S> {
    fn combine(mut self, other: Self) -> Self {
        self.effects.extend(other.effects);
        self
    }
}

impl<S: State> From<DestinedEffect<S>> for EffectOps<S> {
    fn from(effect: DestinedEffect<S>) -> Self {
        EffectOps {
            effects: vec![effect],
        }
    }
}

impl<S: State> From<Vec<DestinedEffect<S>>> for EffectOps<S> {
    fn from(effects: Vec<DestinedEffect<S>>) -> Self {
        EffectOps { effects }
    }
}
