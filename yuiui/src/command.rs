use futures::future::{BoxFuture, FutureExt as _};
use futures::stream::{BoxStream, Stream, StreamExt as _};
use std::fmt;
use std::future::Future;
use std::time::Duration;
use std::vec;

use crate::cancellation_token::CancellationToken;
use crate::context::StateScope;

#[derive(Debug)]
pub struct Command<T> {
    atoms: Vec<(CommandAtom<T>, Option<CancellationToken>)>,
}

impl<T> Command<T> {
    pub fn none() -> Self {
        Command { atoms: Vec::new() }
    }

    pub fn map<F, U>(self, f: F) -> Command<U>
    where
        F: Fn(T) -> U + Clone + Send + 'static,
        T: 'static,
        U: 'static,
    {
        let atoms = self
            .atoms
            .into_iter()
            .map(move |(command, cancellation_token)| (command.map(f.clone()), cancellation_token))
            .collect();
        Command { atoms }
    }
}

impl<T> From<(CommandAtom<T>, Option<CancellationToken>)> for Command<T> {
    fn from(atom: (CommandAtom<T>, Option<CancellationToken>)) -> Self {
        Self { atoms: vec![atom] }
    }
}

impl<T> From<Vec<(CommandAtom<T>, Option<CancellationToken>)>> for Command<T> {
    fn from(atoms: Vec<(CommandAtom<T>, Option<CancellationToken>)>) -> Self {
        Self { atoms }
    }
}

impl<T> IntoIterator for Command<T> {
    type Item = (CommandAtom<T>, Option<CancellationToken>);

    type IntoIter = vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.atoms.into_iter()
    }
}

pub enum CommandAtom<T> {
    Future(BoxFuture<'static, T>),
    Stream(BoxStream<'static, T>),
    Timeout(Duration, Box<dyn FnOnce() -> T + Send>),
    Interval(Duration, Box<dyn Fn() -> T + Send>),
}

impl<T> CommandAtom<T> {
    pub fn from_future<Future>(future: Future) -> Self
    where
        Future: self::Future<Output = T> + Send + 'static,
    {
        CommandAtom::Future(Box::pin(future))
    }

    pub fn from_stream<Stream>(stream: Stream) -> Self
    where
        Stream: self::Stream<Item = T> + Send + 'static,
    {
        CommandAtom::Stream(Box::pin(stream))
    }

    pub fn delay<F>(duration: Duration, f: F) -> Self
    where
        F: FnOnce() -> T + Send + 'static,
    {
        CommandAtom::Timeout(duration, Box::new(f))
    }

    pub fn every<F>(period: Duration, f: F) -> Self
    where
        F: Fn() -> T + Send + 'static,
    {
        CommandAtom::Interval(period, Box::new(f))
    }

    pub fn map<F, U>(self, f: F) -> CommandAtom<U>
    where
        F: Fn(T) -> U + Clone + Send + 'static,
        T: 'static,
        U: 'static,
    {
        match self {
            Self::Future(future) => CommandAtom::Future(Box::pin(future.map(f))),
            Self::Stream(stream) => CommandAtom::Stream(Box::pin(stream.map(f))),
            Self::Timeout(duration, callback) => {
                CommandAtom::Timeout(duration, Box::new(move || f(callback())))
            }
            Self::Interval(period, callback) => {
                CommandAtom::Interval(period, Box::new(move || f(callback())))
            }
        }
    }
}

impl<M> fmt::Debug for CommandAtom<M>
where
    M: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Future(_) => f.debug_tuple("Future").finish(),
            Self::Stream(_) => f.debug_tuple("Stream").finish(),
            Self::Timeout(duration, _) => f.debug_tuple("Timeout").field(duration).finish(),
            Self::Interval(period, _) => f.debug_tuple("Interval").field(period).finish(),
        }
    }
}

pub trait CommandRuntime<M> {
    fn spawn_command(
        &self,
        command: CommandAtom<M>,
        cancellation_token: Option<CancellationToken>,
        state_scope: StateScope,
    );
}
