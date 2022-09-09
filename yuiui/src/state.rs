use crate::cancellation_token::CancellationToken;
use crate::command::Command;

pub trait State: 'static {
    type Message;

    fn update(&mut self, message: Self::Message) -> Effect<Self::Message>;
}

pub enum Effect<T> {
    Batch(Vec<Effect<T>>),
    Command(Command<T>, Option<CancellationToken>),
    RequestUpdate,
}

impl<T> Effect<T> {
    pub fn nop() -> Effect<T> {
        Effect::Batch(Vec::new())
    }

    pub fn map<F, U>(self, f: F) -> Effect<U>
    where
        F: Fn(T) -> U + Clone + Send + 'static,
        T: 'static,
        U: 'static,
    {
        match self {
            Self::Batch(effects) => Effect::Batch(
                effects
                    .into_iter()
                    .map(|effect| effect.map(f.clone()))
                    .collect(),
            ),
            Self::Command(command, cancellation_token) => {
                Effect::Command(command.map(f), cancellation_token)
            }
            Self::RequestUpdate => Effect::RequestUpdate,
        }
    }
}
