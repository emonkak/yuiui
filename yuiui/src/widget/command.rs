use std::future::Future;
use std::ops::Add;
use std::pin::Pin;
use std::time::Instant;

pub enum Command<Message> {
    Exit,
    Send(Message),
    Perform(Pin<Box<dyn Future<Output = Message>>>),
    RequestIdle(Box<dyn FnOnce(Instant) -> Message>),
    Batch(Vec<Command<Message>>),
}

impl<Message> Add for Command<Message> {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        match (self, other) {
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
