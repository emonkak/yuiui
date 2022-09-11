use slot_vec::{slot_tree, SlotTree};
use std::ops::Deref;

use crate::command::CommandBatch;
use crate::id::{Depth, IdPathBuf};

pub type StateId = slot_tree::NodeId;

pub type StateTree = SlotTree<(IdPathBuf, Depth)>;

pub trait State: 'static {
    type Message;

    fn update(&mut self, message: Self::Message) -> (bool, CommandBatch<Self::Message>);
}

#[derive(Debug, Clone)]
pub struct Store<T> {
    state: T,
    dirty: bool,
    subscription: Option<StateId>,
}

impl<T> Store<T> {
    pub fn new(state: T) -> Self {
        Self {
            state,
            dirty: false,
            subscription: None,
        }
    }

    pub(crate) fn dirty(&self) -> bool {
        self.dirty
    }

    pub(crate) fn mark_clean(&mut self) {
        self.dirty = false;
    }

    pub(crate) fn connect<F: FnOnce() -> StateId>(&mut self, f: F) -> StateId {
        *self.subscription.get_or_insert_with(f)
    }

    pub(crate) fn subscription(&self) -> StateId {
        self.subscription.unwrap_or(StateId::ROOT)
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

    fn update(&mut self, message: Self::Message) -> (bool, CommandBatch<Self::Message>) {
        let (dirty, commands) = self.state.update(message);
        if dirty {
            self.dirty = true;
        }
        (dirty, commands)
    }
}
