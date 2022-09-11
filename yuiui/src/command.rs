use futures::future::{BoxFuture, FutureExt as _};
use futures::stream::{BoxStream, Stream, StreamExt as _};
use std::fmt;
use std::future::Future;
use std::time::Duration;
use std::vec;

use crate::cancellation_token::CancellationToken;
use crate::id::StateTree;

pub enum Command<T> {
    Future(BoxFuture<'static, T>),
    Stream(BoxStream<'static, T>),
    Timeout(Duration, Box<dyn FnOnce() -> T + Send>),
    Interval(Duration, Box<dyn Fn() -> T + Send>),
}

impl<T> Command<T> {
    pub fn from_future<Future>(future: Future) -> Self
    where
        Future: self::Future<Output = T> + Send + 'static,
    {
        Command::Future(Box::pin(future))
    }

    pub fn from_stream<Stream>(stream: Stream) -> Self
    where
        Stream: self::Stream<Item = T> + Send + 'static,
    {
        Command::Stream(Box::pin(stream))
    }

    pub fn delay<F>(duration: Duration, f: F) -> Self
    where
        F: FnOnce() -> T + Send + 'static,
    {
        Command::Timeout(duration, Box::new(f))
    }

    pub fn every<F>(period: Duration, f: F) -> Self
    where
        F: Fn() -> T + Send + 'static,
    {
        Command::Interval(period, Box::new(f))
    }

    pub fn map<F, U>(self, f: F) -> Command<U>
    where
        F: Fn(T) -> U + Clone + Send + 'static,
        T: 'static,
        U: 'static,
    {
        match self {
            Self::Future(future) => Command::Future(Box::pin(future.map(f))),
            Self::Stream(stream) => Command::Stream(Box::pin(stream.map(f))),
            Self::Timeout(duration, callback) => {
                Command::Timeout(duration, Box::new(move || f(callback())))
            }
            Self::Interval(period, callback) => {
                Command::Interval(period, Box::new(move || f(callback())))
            }
        }
    }
}

impl<T> fmt::Debug for Command<T>
where
    T: fmt::Debug,
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

#[derive(Debug)]
pub struct CommandBatch<T> {
    commands: Vec<(Command<T>, Option<CancellationToken>)>,
}

impl<T> CommandBatch<T> {
    pub fn none() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    pub fn map<F, U>(self, f: F) -> CommandBatch<U>
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
        CommandBatch { commands }
    }
}

impl<T> From<(Command<T>, Option<CancellationToken>)> for CommandBatch<T> {
    fn from(command: (Command<T>, Option<CancellationToken>)) -> Self {
        Self {
            commands: vec![command],
        }
    }
}

impl<T> From<Vec<(Command<T>, Option<CancellationToken>)>> for CommandBatch<T> {
    fn from(commands: Vec<(Command<T>, Option<CancellationToken>)>) -> Self {
        Self { commands }
    }
}

impl<T> IntoIterator for CommandBatch<T> {
    type Item = (Command<T>, Option<CancellationToken>);

    type IntoIter = vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.commands.into_iter()
    }
}

pub trait ExecutionContext<M> {
    fn spawn_command(
        &self,
        command: Command<M>,
        cancellation_token: Option<CancellationToken>,
        state_tree: StateTree,
    );
}
