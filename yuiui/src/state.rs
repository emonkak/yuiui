use std::ops::Deref;

use crate::cancellation_token::CancellationToken;
use crate::command::Command;
use crate::id::{Depth, IdPath, IdPathBuf};

pub trait State: 'static {
    type Message;

    fn update(&mut self, message: Self::Message) -> (bool, Effect<Self::Message>);
}

#[derive(Debug, Clone)]
pub struct Store<T> {
    state: T,
    dirty: bool,
    subscriptions: Vec<(IdPathBuf, Depth)>,
}

impl<T> Store<T> {
    pub fn new(state: T) -> Self {
        Self {
            state,
            dirty: false,
            subscriptions: Vec::new(),
        }
    }

    pub(crate) fn mark_clean(&mut self) {
        self.dirty = false;
    }

    pub(crate) fn connect(&mut self, id_path: IdPathBuf, depth: Depth) {
        self.subscriptions.push((id_path, depth))
    }

    pub(crate) fn disconnect(&mut self, id_path: &IdPath, depth: Depth) {
        if let Some(position) = self
            .subscriptions
            .iter()
            .position(|(x, y)| x == id_path && *y == depth)
        {
            self.subscriptions.swap_remove(position);
        }
    }

    pub fn dirty(&self) -> bool {
        self.dirty
    }
}

impl<T> Deref for Store<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl<T: State> State for Store<T> {
    type Message = T::Message;

    fn update(&mut self, message: Self::Message) -> (bool, Effect<Self::Message>) {
        let (dirty, mut effect) = self.state.update(message);
        if dirty {
            self.dirty = true;
            effect
                .subscriptions
                .extend(self.subscriptions.iter().cloned());
        }
        (dirty, effect)
    }
}

#[derive(Debug)]
pub struct Effect<T> {
    pub(crate) commands: Vec<(Command<T>, Option<CancellationToken>)>,
    pub(crate) subscriptions: Vec<(IdPathBuf, Depth)>,
}

impl<T> Effect<T> {
    pub fn none() -> Self {
        Self {
            commands: Vec::new(),
            subscriptions: Vec::new(),
        }
    }

    pub fn append(&mut self, other: &mut Effect<T>) {
        self.commands.append(&mut other.commands);
        self.subscriptions.append(&mut other.subscriptions);
    }

    pub fn push_command(
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
            subscriptions: self.subscriptions,
        }
    }
}

impl<T> From<(Command<T>, Option<CancellationToken>)> for Effect<T> {
    fn from(command: (Command<T>, Option<CancellationToken>)) -> Self {
        Self {
            commands: vec![command],
            subscriptions: Vec::new(),
        }
    }
}

impl<T> From<Vec<(Command<T>, Option<CancellationToken>)>> for Effect<T> {
    fn from(commands: Vec<(Command<T>, Option<CancellationToken>)>) -> Self {
        Self {
            commands,
            subscriptions: Vec::new(),
        }
    }
}

impl<T> Extend<Self> for Effect<T> {
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = Self>,
    {
        for effect in iter {
            self.commands.extend(effect.commands);
            self.subscriptions.extend(effect.subscriptions);
        }
    }
}
