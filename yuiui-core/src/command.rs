use futures::future::{BoxFuture, FutureExt as _};
use futures::stream::{BoxStream, Stream, StreamExt as _};
use std::fmt;
use std::future::Future;
use std::time::Duration;

use crate::cancellation_token::CancellationToken;

pub enum Command<T> {
    Future(BoxFuture<'static, T>),
    Stream(BoxStream<'static, T>),
    Timeout(Duration, Box<dyn FnOnce() -> T + Send>),
    Interval(Duration, Box<dyn FnMut() -> T + Send>),
}

impl<T> Command<T> {
    pub fn from_future<Future>(future: Future) -> Self
    where
        Future: self::Future<Output = T> + Send + 'static,
    {
        Self::Future(Box::pin(future))
    }

    pub fn from_stream<Stream>(stream: Stream) -> Self
    where
        Stream: self::Stream<Item = T> + Send + 'static,
    {
        Self::Stream(Box::pin(stream))
    }

    pub fn delay<F>(duration: Duration, f: F) -> Self
    where
        F: FnOnce() -> T + Send + 'static,
    {
        Self::Timeout(duration, Box::new(f))
    }

    pub fn every<F>(period: Duration, f: F) -> Self
    where
        F: FnMut() -> T + Send + 'static,
    {
        Self::Interval(period, Box::new(f))
    }
    pub fn map<F, U>(self, mut f: F) -> Command<U>
    where
        F: FnMut(T) -> U + Send + 'static,
        T: 'static,
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
