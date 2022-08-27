use futures::future::{BoxFuture, FutureExt as _};
use futures::stream::{BoxStream, Stream, StreamExt as _};
use std::collections::{hash_map, HashMap};
use std::future::Future;
use std::mem;

use crate::effect::{Effect, EffectPath};
use crate::render::NodeId;
use crate::state::State;

pub type CommandId = usize;

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

    pub fn map<F, PS>(self, f: F) -> Command<PS>
    where
        S: 'static,
        F: Fn(Effect<S>) -> Effect<PS> + Send + 'static,
        PS: State,
    {
        match self {
            Command::Future(future) => Command::Future(Box::pin(future.map(f))),
            Command::Stream(stream) => Command::Stream(Box::pin(stream.map(f))),
        }
    }
}

pub trait CommandRuntime {
    type Token;

    fn run<S: State>(&mut self, path: EffectPath, command: Command<S>) -> Self::Token;

    fn abort(&mut self, token: Self::Token);
}

pub struct CommandHandler<R: CommandRuntime> {
    runtime: R,
    running_commands: HashMap<NodeId, TokenMap<R::Token>>,
}

impl<R: CommandRuntime> CommandHandler<R> {
    pub fn new(runtime: R) -> Self {
        Self {
            runtime,
            running_commands: HashMap::new(),
        }
    }

    pub fn run<S: State>(
        &mut self,
        path: EffectPath,
        command: Command<S>,
        command_id: Option<CommandId>,
    ) {
        let node_id = path.source_path().as_node_id();
        let token = self.runtime.run(path, command);
        match self.running_commands.entry(node_id) {
            hash_map::Entry::Occupied(mut entry) => {
                if let Some(token) = entry.get_mut().add(token, command_id) {
                    self.runtime.abort(token);
                }
            }
            hash_map::Entry::Vacant(entry) => {
                let mut token_map = TokenMap::new();
                token_map.add(token, command_id);
                entry.insert(token_map);
            }
        }
    }

    pub fn abort(&mut self, node_id: NodeId, command_id: CommandId) -> bool {
        if let Some(token_map) = self.running_commands.get_mut(&node_id) {
            if let Some(token) = token_map.remove(command_id) {
                self.runtime.abort(token);
                return true;
            }
        }
        false
    }

    pub fn abort_all(&mut self, node_id: NodeId) {
        if let Some(token_map) = self.running_commands.remove(&node_id) {
            for token in token_map.tokens {
                self.runtime.abort(token);
            }
            for (token, _) in token_map.identified_tokens {
                self.runtime.abort(token);
            }
        }
    }
}

struct TokenMap<T> {
    tokens: Vec<T>,
    identified_tokens: Vec<(T, CommandId)>,
}

impl<T> TokenMap<T> {
    fn new() -> Self {
        Self {
            tokens: Vec::new(),
            identified_tokens: Vec::new(),
        }
    }

    fn add(&mut self, new_token: T, new_command_id: Option<CommandId>) -> Option<T> {
        match new_command_id {
            Some(new_command_id) => {
                if let Some((token, _)) = self
                    .identified_tokens
                    .iter_mut()
                    .find(|(_, command_id)| *command_id == new_command_id)
                {
                    Some(mem::replace(token, new_token))
                } else {
                    self.identified_tokens.push((new_token, new_command_id));
                    None
                }
            }
            None => {
                self.tokens.push(new_token);
                None
            }
        }
    }

    fn remove(&mut self, command_id: CommandId) -> Option<T> {
        if let Some(index) = self
            .identified_tokens
            .iter()
            .position(|(_, id)| *id == command_id)
        {
            let (token, _) = self.identified_tokens.swap_remove(index);
            Some(token)
        } else {
            None
        }
    }
}
