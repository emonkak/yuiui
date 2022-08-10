pub trait State {
    type Message;

    fn reduce(&mut self, message: Self::Message) -> bool;
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Data<T> {
    pub value: T,
}

impl<T> From<T> for Data<T> {
    fn from(value: T) -> Self {
        Self { value }
    }
}

impl<T: PartialEq> State for Data<T> {
    type Message = T;

    fn reduce(&mut self, message: Self::Message) -> bool {
        if &self.value != &message {
            self.value = message;
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
