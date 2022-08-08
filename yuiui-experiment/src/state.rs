pub trait State {
    type Message;

    fn reduce(&mut self, message: Self::Message) -> bool;
}

impl State for () {
    type Message = ();

    fn reduce(&mut self, _message: Self::Message) -> bool {
        false
    }
}

#[derive(Debug)]
pub struct Immediate<T> {
    pub value: T,
}

impl<T> From<T> for Immediate<T> {
    fn from(value: T) -> Self {
        Self { value }
    }
}

impl<T: PartialEq> State for Immediate<T> {
    type Message = T;

    fn reduce(&mut self, message: Self::Message) -> bool {
        if self.value != message {
            self.value = message;
            true
        } else {
            false
        }
    }
}
