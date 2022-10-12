use crate::cancellation_token::CancellationToken;
use crate::command::Command;
use crate::id::{Depth, IdPathBuf};

#[derive(Debug, Default)]
pub struct Effect<T> {
    pub(crate) commands: Vec<(Command<T>, Option<CancellationToken>)>,
    pub(crate) subscribers: Vec<(IdPathBuf, Depth)>,
}

impl<T> Effect<T> {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
            subscribers: Vec::new(),
        }
    }

    pub fn add_command(
        &mut self,
        command: Command<T>,
        cancellation_token: Option<CancellationToken>,
    ) {
        self.commands.push((command, cancellation_token));
    }

    pub fn map<F, U>(self, f: F) -> Effect<U>
    where
        F: Fn(T) -> U + Clone + Send + 'static,
        T: 'static,
        U: 'static,
    {
        let commands = self
            .commands
            .into_iter()
            .map(move |(command, cancellation_token)| (command.map(f.clone()), cancellation_token))
            .collect();
        Effect {
            commands,
            subscribers: self.subscribers,
        }
    }

    pub fn append(&mut self, other: &mut Effect<T>) {
        self.commands.append(&mut other.commands);
        self.subscribers.append(&mut other.subscribers);
    }
}
