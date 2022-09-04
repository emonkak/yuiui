pub trait State: Send + 'static {
    type Message: Send;

    fn reduce(&mut self, message: Self::Message) -> bool;
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Versioned<T> {
    state: T,
    version: u64,
}

impl<T> From<T> for Versioned<T> {
    fn from(state: T) -> Self {
        Self { state, version: 0 }
    }
}

impl<T: State> State for Versioned<T> {
    type Message = T::Message;

    fn reduce(&mut self, message: Self::Message) -> bool {
        if self.state.reduce(message) {
            self.version += 1;
            true
        } else {
            false
        }
    }
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

impl<T: Send + PartialEq + 'static> State for Data<T> {
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
