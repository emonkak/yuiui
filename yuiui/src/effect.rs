use std::future::Future;
use std::pin::Pin;

use crate::state::State;

pub type BoxFuture<T> = Pin<Box<dyn Future<Output = T>>>;

pub enum Effect<S: State> {
    Message(S::Message),
    Mutation(Box<dyn Mutation<S>>),
    Command(BoxFuture<Effect<S>>),
}

pub trait Mutation<S> {
    fn apply(&mut self, state: &mut S) -> bool;
}

impl<S: State> Mutation<S> for Box<dyn Mutation<S>> {
    fn apply(&mut self, state: &mut S) -> bool {
        self.as_mut().apply(state)
    }
}

impl<S: State> Mutation<S> for Option<S::Message> {
    fn apply(&mut self, state: &mut S) -> bool {
        state.reduce(self.take().unwrap())
    }
}
