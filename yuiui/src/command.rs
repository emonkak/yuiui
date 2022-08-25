use futures::stream::{BoxStream, Stream, StreamExt};

use crate::message::Message;
use crate::state::State;

pub struct Command<S: State> {
    stream: BoxStream<'static, Message<S>>,
}

impl<S: State> Command<S> {
    pub fn new(stream: BoxStream<'static, Message<S>>) -> Command<S> {
        Self { stream }
    }

    pub fn into_stream(self) -> BoxStream<'static, Message<S>> {
        self.stream
    }
}

impl<Stream, S> From<Stream> for Command<S>
where
    Stream: self::Stream<Item = Message<S>> + Send + 'static,
    S: State,
{
    fn from(stream: Stream) -> Self {
        Command::new(Box::pin(stream))
    }
}

impl<S: State> Command<S> {
    pub fn map<F, PS>(self, f: F) -> Command<PS>
    where
        F: Fn(Message<S>) -> Message<PS> + Send + 'static,
        S: 'static,
        PS: State,
    {
        Command {
            stream: Box::pin(self.stream.map(f)),
        }
    }
}

pub trait CommandHandler<S: State> {
    type Token;

    fn run(&mut self, command: Command<S>) -> Self::Token;

    fn cancel(&mut self, token: Self::Token);
}
