use std::collections::HashMap;
use std::ops::Deref;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::command::CommandBatch;
use crate::id::{Depth, Id, IdPath, IdPathBuf};

pub trait State: 'static {
    type Message;

    fn update(&mut self, message: Self::Message) -> (bool, CommandBatch<Self::Message>);
}

#[derive(Debug)]
pub struct Store<T> {
    state: T,
    dirty: AtomicBool,
    subscribers: HashMap<(Id, Depth), IdPathBuf>,
}

impl<T> Store<T> {
    pub fn new(state: T) -> Self {
        Self {
            state,
            dirty: AtomicBool::new(false),
            subscribers: HashMap::new(),
        }
    }

    pub(crate) fn dirty(&self) -> bool {
        self.dirty.load(Ordering::Relaxed)
    }

    pub(crate) fn mark_dirty(&self) {
        self.dirty.store(true, Ordering::Relaxed)
    }

    pub(crate) fn mark_clean(&self) {
        self.dirty.store(false, Ordering::Relaxed)
    }

    pub(crate) fn to_subscribers(&self) -> Vec<(IdPathBuf, Depth)> {
        self.subscribers
            .iter()
            .map(|((_, depth), id_path)| (id_path.to_vec(), *depth))
            .collect()
    }

    pub(crate) fn subscribe(&mut self, id_path: IdPathBuf, depth: Depth) {
        self.subscribers
            .insert((Id::from_bottom(&id_path), depth), id_path);
    }

    pub(crate) fn unsubscribe(&mut self, id_path: &IdPath, depth: Depth) {
        self.subscribers.remove(&(Id::from_bottom(&id_path), depth));
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
            self.mark_dirty();
        }
        (dirty, commands)
    }
}
