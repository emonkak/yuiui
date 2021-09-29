use std::ops::Add;
use yuiui_support::bit_flags::BitFlags;

use super::{Command, EventMask};

pub enum Effect<Message> {
    None,
    AddListener(BitFlags<EventMask>),
    RemoveListener(BitFlags<EventMask>),
    Command(Command<Message>),
    Batch(Vec<Effect<Message>>),
}

impl<Message, Rhs: Into<Effect<Message>>> Add<Rhs> for Effect<Message> {
    type Output = Self;

    fn add(self, other: Rhs) -> Self::Output {
        match (self, other.into()) {
            (Self::None, y) => y,
            (x, Self::None) => x,
            (Self::Batch(mut xs), Self::Batch(ys)) => {
                xs.extend(ys);
                Self::Batch(xs)
            }
            (Self::Batch(mut xs), y) => {
                xs.push(y);
                Self::Batch(xs)
            }
            (x, Self::Batch(ys)) => {
                let mut xs = vec![x];
                xs.extend(ys);
                Self::Batch(xs)
            }
            (x, y) => Self::Batch(vec![x, y]),
        }
    }
}

impl<Message> From<Command<Message>> for Effect<Message> {
    fn from(command: Command<Message>) -> Self {
        Self::Command(command)
    }
}

impl<Message> From<Vec<Effect<Message>>> for Effect<Message> {
    fn from(effects: Vec<Effect<Message>>) -> Self {
        Self::Batch(effects)
    }
}
