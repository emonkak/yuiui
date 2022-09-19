use crate::effect::Effect;
use crate::id::{Depth, IdPath, IdPathBuf};

pub trait State {
    type Message;

    fn update(&mut self, message: Self::Message) -> (bool, Effect<Self::Message>);
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

    pub(crate) fn subscribe(&mut self, id_path: IdPathBuf, depth: Depth) {
        self.subscribers.push((id_path, depth))
    }

    pub(crate) fn unsubscribe(&mut self, id_path: &IdPath, depth: Depth) {
        if let Some(position) = self
            .subscribers
            .iter()
            .position(|(x, y)| x == id_path && *y == depth)
        {
            self.subscribers.swap_remove(position);
        }
    }

    pub(crate) fn mark_clean(&mut self) {
        self.dirty = false;
    }

    pub(crate) fn state(&self) -> &T {
        &self.state
    }

    pub(crate) fn dirty(&self) -> bool {
        self.dirty
    }
}

impl<T: State> State for Store<T> {
    type Message = T::Message;

    fn update(&mut self, message: Self::Message) -> (bool, Effect<Self::Message>) {
        let (dirty, mut effect) = self.state.update(message);
        if dirty {
            self.dirty = true;
            effect.subscribers.extend(self.subscribers.iter().cloned());
        }
        (dirty, effect)
    }
}
