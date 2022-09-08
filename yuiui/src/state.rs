pub trait State: 'static {
    type Message;

    fn reduce(&mut self, message: Self::Message) -> bool;
}
