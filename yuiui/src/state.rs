use std::cell::RefCell;
use std::mem;

use crate::effect::Effect;
use crate::id::{Depth, IdPath, Subscriber};

pub trait State {
    type Message;

    fn update(&mut self, message: Self::Message) -> Effect<Self::Message>;
}

#[derive(Clone, Debug, Default)]
pub struct Atom<T> {
    value: T,
    subscribers: RefCell<Vec<Subscriber>>,
}

impl<T> Atom<T> {
    #[inline]
    pub fn new(value: T) -> Self {
        Self {
            value,
            subscribers: RefCell::new(Vec::new()),
        }
    }

    #[inline]
    pub fn update<F>(&mut self, f: F) -> Vec<Subscriber>
    where
        F: FnOnce(&mut T),
    {
        f(&mut self.value);
        mem::take(self.subscribers.get_mut())
    }

    #[inline]
    pub fn peek(&self) -> &T {
        &self.value
    }

    pub(crate) fn subscribe(&self, id_path: &IdPath, depth: Depth) {
        for subscriber in self.subscribers.borrow_mut().iter_mut() {
            if subscriber.id_path == id_path {
                if subscriber.depth < depth {
                    subscriber.depth = depth;
                }
                return;
            }
        }
        self.subscribers.borrow_mut().push(Subscriber {
            id_path: id_path.to_vec(),
            depth,
        });
    }
}
