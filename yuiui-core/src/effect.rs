use crate::cancellation_token::CancellationToken;
use crate::command::Command;
use crate::id::NodePath;

#[derive(Debug)]
pub enum Effect<T> {
    Command(Command<T>, Option<CancellationToken>),
    Update(Vec<NodePath>),
    ForceUpdate,
    Batch(Vec<Effect<T>>),
}

impl<T> Effect<T> {
    pub fn nop() -> Self {
        Self::Batch(Vec::new())
    }

    pub fn map<F, U>(self, f: F) -> Effect<U>
    where
        F: FnMut(T) -> U + Clone + Send + 'static,
        T: 'static,
        U: 'static,
    {
        match self {
            Effect::Command(command, cancellation_token) => {
                Effect::Command(command.map(f.clone()), cancellation_token)
            }
            Effect::Update(subscribers) => Effect::Update(subscribers),
            Effect::ForceUpdate => Effect::ForceUpdate,
            Effect::Batch(effects) => Effect::Batch(
                effects
                    .into_iter()
                    .map(|effect| effect.map(f.clone()))
                    .collect(),
            ),
        }
    }

    pub fn compose(self, rhs: Effect<T>) -> Self {
        match (self, rhs) {
            (Effect::Batch(mut lhs), Effect::Batch(rhs)) => {
                lhs.extend(rhs);
                Effect::Batch(lhs)
            }
            (Effect::Batch(mut lhs), rhs) => {
                lhs.push(rhs);
                Effect::Batch(lhs)
            }
            (lhs, Effect::Batch(rhs)) => {
                let mut effects = Vec::with_capacity(rhs.len() + 1);
                effects.push(lhs);
                effects.extend(rhs);
                Effect::Batch(effects)
            }
            (lhs, rhs) => Effect::Batch(vec![lhs, rhs]),
        }
    }
}
