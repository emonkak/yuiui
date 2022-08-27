pub trait State: Send + 'static {
    type Message: Send;

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
