use std::future::Future;
use std::pin::Pin;
use std::time::Instant;

pub enum Command<Message> {
    None,
    Exit,
    Identity(Message),
    Perform(Pin<Box<dyn Future<Output = Message> + 'static>>),
    RequestIdle(Box<dyn FnOnce(Instant) -> Message>),
    Batch(Vec<Command<Message>>),
}

impl<Message> Command<Message> {
    pub fn compose(self, other: Self) -> Self {
        match (self, other) {
            (Command::None, y) => y,
            (x, Command::None) => x,
            (Command::Batch(mut xs), Command::Batch(ys)) => {
                xs.extend(ys);
                Command::Batch(xs)
            }
            (Command::Batch(mut xs), y) => {
                xs.push(y);
                Command::Batch(xs)
            }
            (x, Command::Batch(ys)) => {
                let mut xs = vec![x];
                xs.extend(ys);
                Command::Batch(xs)
            }
            (x, y) => Command::Batch(vec![x, y]),
        }
    }
}
