use std::ops::Deref;

use crate::command::CommandBatch;
use crate::id::{Depth, IdPath, IdPathBuf};

pub trait State: 'static {
    type Message;

    fn update(&mut self, message: Self::Message) -> (bool, CommandBatch<Self::Message>);
}

#[derive(Debug, Clone)]
pub struct Store<T> {
    state: T,
    dirty: bool,
    subscribers: Vec<(IdPathBuf, Depth)>,
}

impl<T> Store<T> {
    pub fn new(state: T) -> Self {
        Self {
            state,
            dirty: false,
            subscribers: Vec::new(),
        }
    }

    pub(crate) fn dirty(&self) -> bool {
        self.dirty
    }

    pub(crate) fn mark_clean(&mut self) {
        self.dirty = false;
    }

    pub(crate) fn add_subscriber(&mut self, id_path: IdPathBuf, depth: Depth) {
        self.subscribers.push((id_path, depth))
    }

    pub(crate) fn remove_subscriber(&mut self, id_path: &IdPath, depth: Depth) {
        if let Some(position) =
            self.subscribers
                .iter()
                .position(|(existing_id_path, existing_depth)| {
                    existing_id_path.last() == id_path.last() && *existing_depth == depth
                })
        {
            self.subscribers.swap_remove(position);
        }
    }

    pub(crate) fn to_subscribers(&self) -> Vec<(IdPathBuf, Depth)> {
        self.subscribers.to_vec()
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
