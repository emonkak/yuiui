use futures::future::{BoxFuture, FutureExt as _};
use futures::stream::{BoxStream, Stream, StreamExt as _};
use std::future::Future;

use crate::effect::{Effect, EffectPath};
use crate::state::State;

pub enum Command<S: State> {
    Future(BoxFuture<'static, Effect<S>>),
    Stream(BoxStream<'static, Effect<S>>),
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
        }
    }
}

pub trait CommandHandler {
    fn run<S: State>(&mut self, path: EffectPath, command: Command<S>);
}
