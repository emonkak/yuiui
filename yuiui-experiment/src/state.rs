pub trait State {
    type Message;

    fn reduce(&mut self, _message: Self::Message) -> bool;
}

impl<T: PartialEq> State for T {
    type Message = T;

    fn reduce(&mut self, message: Self::Message) -> bool {
        if self != &message {
            *self = message;
            true
        } else {
            false
        }
    }
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

pub enum Effect<S: State> {
    Message(S::Message),
    Mutation(Box<dyn Mutation<S>>),
}
