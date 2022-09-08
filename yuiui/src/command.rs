use futures::future::{BoxFuture, FutureExt as _};
use futures::stream::{BoxStream, Stream, StreamExt as _};
use std::fmt;
use std::future::Future;
use std::time::Duration;

use crate::effect::Effect;

pub enum Command<M> {
    Future(BoxFuture<'static, Effect<M>>),
    Stream(BoxStream<'static, Effect<M>>),
    Timeout(Duration, Box<dyn FnOnce() -> Effect<M> + Send>),
    Interval(Duration, Box<dyn Fn() -> Effect<M> + Send>),
}

impl<M> Command<M> {
    pub fn from_future<Future>(future: Future) -> Self
    where
        Future: self::Future<Output = Effect<M>> + Send + 'static,
    {
        Command::Future(Box::pin(future))
    }

    pub fn from_stream<Stream>(stream: Stream) -> Self
    where
        Stream: self::Stream<Item = Effect<M>> + Send + 'static,
    {
        Command::Stream(Box::pin(stream))
    }

    pub fn delay<F>(duration: Duration, f: F) -> Self
    where
        F: FnOnce() -> Effect<M> + Send + 'static,
    {
        Command::Timeout(duration, Box::new(f))
    }

    pub fn every<F>(period: Duration, f: F) -> Self
    where
        F: Fn() -> Effect<M> + Send + 'static,
    {
        Command::Interval(period, Box::new(f))
    }

    pub fn map<F, N>(self, f: F) -> Command<N>
    where
        F: Fn(Effect<M>) -> Effect<N> + Send + 'static,
        M: 'static,
        N: 'static,
    {
        match self {
            Command::Future(future) => Command::Future(Box::pin(future.map(f))),
            Command::Stream(stream) => Command::Stream(Box::pin(stream.map(f))),
            Command::Timeout(duration, callback) => {
                Command::Timeout(duration, Box::new(move || f(callback())))
            }
            Command::Interval(period, callback) => {
                Command::Interval(period, Box::new(move || f(callback())))
            }
        }
    }
}

impl<M> fmt::Debug for Command<M>
where
    M: fmt::Debug,
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
