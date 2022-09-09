use std::ops::Deref;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::command::Command;

pub trait State: 'static {
    type Message;

    fn update(&mut self, message: Self::Message) -> (bool, Command<Self::Message>);
}

#[derive(Debug)]
pub struct Store<T> {
    state: T,
    dirty: AtomicBool,
}

impl<T> Store<T> {
    pub fn new(state: T) -> Self {
        Self {
            state,
            dirty: AtomicBool::new(false),
        }
    }

    pub(crate) fn dirty(&self) -> bool {
        self.dirty.load(Ordering::Relaxed)
    }

    pub(crate) fn mark_clean(&self) {
        self.dirty.store(false, Ordering::Relaxed)
    }
}

impl<T> Deref for Store<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl<T: State> State for Store<T> {
    type Message = T::Message;

    fn update(&mut self, message: Self::Message) -> (bool, Command<Self::Message>) {
        let (dirty, commands) = self.state.update(message);
        if dirty {
            self.dirty.store(true, Ordering::Relaxed)
        }
        (dirty, commands)
    }
}
