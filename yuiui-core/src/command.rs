use futures::future::{BoxFuture, FutureExt as _};
use futures::stream::{BoxStream, Stream, StreamExt as _};
use std::fmt;
use std::future::Future;
use std::time::Duration;

use crate::cancellation_token::CancellationToken;

#[derive(Debug)]
pub struct CancellableCommand<T> {
    pub(crate) command: Command<T>,
    pub(crate) cancellation_token: Option<CancellationToken>,
}

impl<T> CancellableCommand<T> {
    pub fn new(command: Command<T>, cancellation_token: Option<CancellationToken>) -> Self {
        Self {
            command,
            cancellation_token,
        }
    }

    pub fn from_future<Future>(
        future: Future,
        cancellation_token: Option<CancellationToken>,
    ) -> Self
    where
        Future: self::Future<Output = T> + Send + 'static,
    {
        Self::new(Command::Future(Box::pin(future)), cancellation_token)
    }

    pub fn from_stream<Stream>(
        stream: Stream,
        cancellation_token: Option<CancellationToken>,
    ) -> Self
    where
        Stream: self::Stream<Item = T> + Send + 'static,
    {
        Self::new(Command::Stream(Box::pin(stream)), cancellation_token)
    }

    pub fn delay<F>(duration: Duration, f: F, cancellation_token: Option<CancellationToken>) -> Self
    where
        F: FnOnce() -> T + Send + 'static,
    {
        Self::new(Command::Timeout(duration, Box::new(f)), cancellation_token)
    }

    pub fn every<F>(period: Duration, f: F, cancellation_token: Option<CancellationToken>) -> Self
    where
        F: FnMut() -> T + Send + 'static,
    {
        Self::new(Command::Interval(period, Box::new(f)), cancellation_token)
    }

    pub fn map<F, U>(self, f: F) -> CancellableCommand<U>
    where
        F: FnMut(T) -> U + Send + 'static,
        T: 'static,
        U: 'static,
    {
        CancellableCommand {
            command: self.command.map(f),
            cancellation_token: self.cancellation_token,
        }
    }
}

pub enum Command<T> {
    Future(BoxFuture<'static, T>),
    Stream(BoxStream<'static, T>),
    Timeout(Duration, Box<dyn FnOnce() -> T + Send>),
    Interval(Duration, Box<dyn FnMut() -> T + Send>),
}

impl<T> Command<T> {
    pub fn map<F, U>(self, mut f: F) -> Command<U>
    where
        F: FnMut(T) -> U + Send + 'static,
        T: 'static,
        U: 'static,
    {
        match self {
            Self::Future(future) => Command::Future(Box::pin(future.map(f))),
            Self::Stream(stream) => Command::Stream(Box::pin(stream.map(f))),
            Self::Timeout(duration, callback) => {
                Command::Timeout(duration, Box::new(move || f(callback())))
            }
            Self::Interval(period, mut callback) => {
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
            Self::Future(_) => f.debug_struct("Future").finish_non_exhaustive(),
            Self::Stream(_) => f.debug_struct("Stream").finish_non_exhaustive(),
            Self::Timeout(duration, _) => f
                .debug_struct("Timeout")
                .field("duration", duration)
                .finish_non_exhaustive(),
            Self::Interval(period, _) => f
                .debug_struct("Interval")
                .field("period", period)
                .finish_non_exhaustive(),
        }
    }
}

pub trait CommandRuntime<T> {
    fn spawn_command(&self, command: Command<T>, cancellation_token: Option<CancellationToken>);
}
