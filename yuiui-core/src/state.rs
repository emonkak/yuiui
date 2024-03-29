use std::cell::RefCell;
use std::mem;

use crate::id::{IdPath, Level, NodePath};

pub trait State {
    type Message;

    fn update(&mut self, message: Self::Message) -> Effect;
}

#[derive(Debug)]
pub enum Effect {
    Nop,
    Update(Vec<NodePath>),
    ForceUpdate,
}

impl Effect {
    pub fn compose(self, rhs: Self) -> Self {
        match (self, rhs) {
            (Self::Nop, rhs) => rhs,
            (lhs, Self::Nop) => lhs,
            (Self::ForceUpdate, _) | (_, Self::ForceUpdate) => Self::ForceUpdate,
            (Self::Update(mut lhs), Self::Update(rhs)) => {
                lhs.extend(rhs);
                Effect::Update(lhs)
            }
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Atom<T> {
    value: T,
    subscribers: RefCell<Vec<NodePath>>,
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
    pub fn get(&self) -> &T {
        &self.value
    }

    #[inline]
    pub fn set(&mut self, new_value: T) -> Effect {
        self.value = new_value;
        Effect::Update(mem::take(self.subscribers.get_mut()))
    }

    #[inline]
    pub fn update<F>(&mut self, f: F) -> Effect
    where
        F: FnOnce(&mut T),
    {
        f(&mut self.value);
        Effect::Update(mem::take(self.subscribers.get_mut()))
    }

    #[inline]
    pub fn update_if<F>(&mut self, f: F) -> Effect
    where
        F: FnOnce(&mut T) -> bool,
    {
        if f(&mut self.value) {
            Effect::Update(mem::take(self.subscribers.get_mut()))
        } else {
            Effect::Nop
        }
    }

    pub(crate) fn subscribe(&self, id_path: &IdPath, level: Level) {
        for subscriber in self.subscribers.borrow_mut().iter_mut() {
            if subscriber.id_path == id_path {
                if subscriber.level < level {
                    subscriber.level = level;
                }
                return;
            }
        }
        self.subscribers.borrow_mut().push(NodePath {
            id_path: id_path.to_vec(),
            level,
        });
    }
}
