use crate::cancellation_token::CancellationToken;
use crate::command::Command;
use crate::id::{Depth, IdPath, IdPathBuf};

pub trait State {
    type Message;

    fn update(&mut self, message: Self::Message) -> (bool, Effect<Self::Message>);
}

#[derive(Debug, Clone)]
pub struct Store<T> {
    state: T,
    dirty: bool,
    subscribers: Vec<(IdPathBuf, Depth)>,
}

impl<T> Store<T> {
    pub fn new(state: T) -> Self {
        Self {
            state,
            dirty: false,
            subscribers: Vec::new(),
        }
    }

    pub(crate) fn subscribe(&mut self, id_path: IdPathBuf, depth: Depth) {
        self.subscribers.push((id_path, depth))
    }

    pub(crate) fn unsubscribe(&mut self, id_path: &IdPath, depth: Depth) {
        if let Some(position) = self
            .subscribers
            .iter()
            .position(|(x, y)| x == id_path && *y == depth)
        {
            self.subscribers.swap_remove(position);
        }
    }

    pub(crate) fn mark_clean(&mut self) {
        self.dirty = false;
    }

    pub(crate) fn state(&self) -> &T {
        &self.state
    }

    pub(crate) fn dirty(&self) -> bool {
        self.dirty
    }
}

impl<T: State> State for Store<T> {
    type Message = T::Message;

    fn update(&mut self, message: Self::Message) -> (bool, Effect<Self::Message>) {
        let (dirty, mut effect) = self.state.update(message);
        if dirty {
            self.dirty = true;
            effect.subscribers.extend(self.subscribers.iter().cloned());
        }
        (dirty, effect)
    }
}

#[derive(Debug)]
pub struct Effect<T> {
    pub(crate) commands: Vec<(Command<T>, Option<CancellationToken>)>,
    pub(crate) subscribers: Vec<(IdPathBuf, Depth)>,
}

impl<T> Effect<T> {
    pub fn none() -> Self {
        Self {
            commands: Vec::new(),
            subscribers: Vec::new(),
        }
    }

    pub fn append(&mut self, other: &mut Effect<T>) {
        self.commands.append(&mut other.commands);
        self.subscribers.append(&mut other.subscribers);
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
}

impl<T> From<(Command<T>, Option<CancellationToken>)> for Effect<T> {
    fn from(command: (Command<T>, Option<CancellationToken>)) -> Self {
        Self {
            commands: vec![command],
            subscribers: Vec::new(),
        }
    }
}

impl<T> From<Vec<(Command<T>, Option<CancellationToken>)>> for Effect<T> {
    fn from(commands: Vec<(Command<T>, Option<CancellationToken>)>) -> Self {
        Self {
            commands,
            subscribers: Vec::new(),
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
            self.subscribers.extend(effect.subscribers);
        }
    }
}
