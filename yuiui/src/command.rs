use futures::stream::{Stream, BoxStream, StreamExt};

use crate::effect::Effect;
use crate::state::State;

pub struct Command<S: State> {
    stream: BoxStream<'static, Effect<S>>,
}

impl<S: State> Command<S> {
    pub fn new(stream: BoxStream<'static, Effect<S>>) -> Command<S> {
        Self {
            stream
        }
    }

    pub fn into_stream(self) -> BoxStream<'static, Effect<S>> {
        self.stream
    }
}

impl<Stream, S> From<Stream> for Command<S>
where
    Stream: self::Stream<Item = Effect<S>> + Send + 'static,
    S: State,
{
    fn from(stream: Stream) -> Self {
        Command::new(Box::pin(stream))
    }
}

impl<S: State> Command<S> {
    pub fn map<F, NS>(self, f: F) -> Command<NS>
    where
        F: Fn(Effect<S>) -> Effect<NS> + Send + 'static,
        S: 'static,
        NS: State,
    {
        Command {
            stream: Box::pin(self.stream.map(f))
        }
    }
}

pub trait CommandHandler<S: State> {
    type Token;

    fn run(&mut self, command: Command<S>) -> Self::Token;

    fn cancel(&mut self, token: Self::Token);
}
