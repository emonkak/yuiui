use std::cell::{Cell, RefCell};
use std::ops;

use crate::effect::Effect;
use crate::id::{Depth, IdPath, IdPathBuf};

pub trait State {
    type Message;

    fn update(&mut self, message: Self::Message) -> (bool, Effect<Self::Message>);
}

#[derive(Clone, Debug, Default)]
pub struct Store<T> {
    state: T,
    dirty: Cell<bool>,
    subscribers: RefCell<Vec<(IdPathBuf, Depth)>>,
}

impl<T> Store<T> {
    pub fn new(state: T) -> Self {
        Self {
            state,
            dirty: Cell::new(false),
            subscribers: RefCell::new(Vec::new()),
        }
    }

    pub(crate) fn subscribe(&self, id_path: IdPathBuf, depth: Depth) {
        self.subscribers.borrow_mut().push((id_path, depth))
    }

    pub(crate) fn unsubscribe(&self, id_path: &IdPath, depth: Depth) {
        if let Some(position) = self
            .subscribers
            .borrow()
            .iter()
            .position(|(x, y)| x == id_path && *y == depth)
        {
            self.subscribers.borrow_mut().remove(position);
        }
    }

    pub(crate) fn mark_clean(&self) {
        self.dirty.set(false)
    }

    pub(crate) fn dirty(&self) -> bool {
        self.dirty.get()
    }
}

impl<T> ops::Deref for Store<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl<T: State> State for Store<T> {
    type Message = T::Message;

    fn update(&mut self, message: Self::Message) -> (bool, Effect<Self::Message>) {
        let (dirty, mut effect) = self.state.update(message);
        if dirty {
            self.dirty.set(true);
            effect
                .subscribers
                .extend(self.subscribers.get_mut().iter().cloned());
        }
        (dirty, effect)
    }
}
