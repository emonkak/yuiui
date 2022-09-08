use std::fmt;
use std::sync::Arc;

use crate::cancellation_token::CancellationToken;
use crate::command::Command;
use crate::context::{EffectContext, StateScope};
use crate::id::{Depth, IdPathBuf};
use crate::traversable::Monoid;

pub enum Effect<M> {
    Message(M),
    Command(Command<M>, Option<CancellationToken>),
    RequestUpdate,
}

impl<M> Effect<M> {
    pub fn destine(self, context: &EffectContext) -> DestinedEffect<M> {
        match self {
            Self::Message(message) => {
                DestinedEffect::Message(message, context.state_scope().clone())
            }
            Self::Command(command, cancellation_token) => {
                DestinedEffect::Command(command, cancellation_token, context.clone())
            }
            Self::RequestUpdate => {
                DestinedEffect::RequestUpdate(context.id_path().to_vec(), context.depth())
            }
        }
    }

    pub(crate) fn lift<F, N>(self, f: &Arc<F>) -> Effect<N>
    where
        F: Fn(M) -> N + Sync + Send + 'static,
        M: 'static,
        N: 'static,
    {
        match self {
            Self::Message(message) => Effect::Message(f(message)),
            Self::Command(command, cancellation_token) => {
                let f = f.clone();
                let command = command.map(move |effect| effect.lift(&f));
                Effect::Command(command, cancellation_token)
            }
            Self::RequestUpdate => Effect::RequestUpdate,
        }
    }
}

impl<M> fmt::Debug for Effect<M>
where
    M: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Message(message) => f.debug_tuple("Message").field(message).finish(),
            Self::Command(command, cancellation_token) => f
                .debug_tuple("Command")
                .field(command)
                .field(cancellation_token)
                .finish(),
            Self::RequestUpdate => f.debug_tuple("RequestUpdate").finish(),
        }
    }
}

pub enum DestinedEffect<M> {
    Message(M, StateScope),
    Command(Command<M>, Option<CancellationToken>, EffectContext),
    RequestUpdate(IdPathBuf, Depth),
}

impl<M> DestinedEffect<M> {
    pub(crate) fn lift<F, N>(self, f: &Arc<F>) -> DestinedEffect<N>
    where
        F: Fn(M) -> N + Sync + Send + 'static,
        M: 'static,
        N: 'static,
    {
        match self {
            Self::Message(message, state_scope) => DestinedEffect::Message(f(message), state_scope),
            Self::Command(command, cancellation_token, context) => {
                let f = f.clone();
                let command = command.map(move |effect| effect.lift(&f));
                DestinedEffect::Command(command, cancellation_token, context)
            }
            Self::RequestUpdate(id_path, depth) => DestinedEffect::RequestUpdate(id_path, depth),
        }
    }
}

impl<M> fmt::Debug for DestinedEffect<M>
where
    M: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Message(message, state_scope) => f
                .debug_tuple("Message")
                .field(message)
                .field(state_scope)
                .finish(),
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
pub struct EffectOps<M> {
    effects: Vec<DestinedEffect<M>>,
}

impl<M> EffectOps<M> {
    pub fn nop() -> Self {
        EffectOps {
            effects: Vec::new(),
        }
    }

    pub fn into_effects(self) -> Vec<DestinedEffect<M>> {
        self.effects
    }

    pub(crate) fn lift<F, N>(self, f: &Arc<F>) -> EffectOps<N>
    where
        F: Fn(M) -> N + Sync + Send + 'static,
        M: 'static,
        N: 'static,
    {
        let effects = self
            .effects
            .into_iter()
            .map(|effect| effect.lift(f))
            .collect();
        EffectOps { effects }
    }
}

impl<M> Default for EffectOps<M> {
    fn default() -> Self {
        EffectOps::nop()
    }
}

impl<M> Monoid for EffectOps<M> {
    fn combine(mut self, other: Self) -> Self {
        self.effects.extend(other.effects);
        self
    }
}

impl<M> From<DestinedEffect<M>> for EffectOps<M> {
    fn from(effect: DestinedEffect<M>) -> Self {
        EffectOps {
            effects: vec![effect],
        }
    }
}

impl<M> From<Vec<DestinedEffect<M>>> for EffectOps<M> {
    fn from(effects: Vec<DestinedEffect<M>>) -> Self {
        EffectOps { effects }
    }
}
