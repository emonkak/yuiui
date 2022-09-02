use futures::future::{BoxFuture, FutureExt as _};
use futures::stream::{BoxStream, Stream, StreamExt as _};
use std::fmt;
use std::future::Future;
use std::time::Duration;

use crate::effect::{Effect, EffectPath};
use crate::state::State;

pub enum Command<S: State> {
    Future(BoxFuture<'static, Effect<S>>),
    Stream(BoxStream<'static, Effect<S>>),
    Timeout(Duration, Box<dyn FnOnce() -> Effect<S> + Send>),
    Interval(Duration, Box<dyn Fn() -> Effect<S> + Send>),
}

impl<S: State> Command<S> {
    pub fn from_future<Future>(future: Future) -> Self
    where
        Future: self::Future<Output = Effect<S>> + Send + 'static,
    {
        Command::Future(Box::pin(future))
    }

    pub fn from_stream<Stream>(stream: Stream) -> Self
    where
        Stream: self::Stream<Item = Effect<S>> + Send + 'static,
    {
        Command::Stream(Box::pin(stream))
    }

    pub fn map<F, NS>(self, f: F) -> Command<NS>
    where
        F: Fn(Effect<S>) -> Effect<NS> + Send + 'static,
        NS: State,
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

impl<S: State> fmt::Debug for Command<S>
where
    S::Message: fmt::Debug,
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
