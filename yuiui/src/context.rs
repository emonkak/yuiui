use crate::id::{id_counter, Id, IdCounter, IdPath, IdPathBuf};

use std::collections::VecDeque;

pub trait IdContext {
    fn push_id(&mut self, id: Id);

    fn pop_id(&mut self);
}

#[derive(Debug)]
pub struct RenderContext {
    id_path: IdPathBuf,
    id_counter: IdCounter,
    preloaded_ids: VecDeque<Id>,
}

impl RenderContext {
    pub fn new() -> Self {
        Self {
            id_path: IdPathBuf::new(),
            id_counter: IdCounter::new(),
            preloaded_ids: VecDeque::new(),
        }
    }

    pub(crate) fn next_id(&mut self) -> Id {
        self.preloaded_ids
            .pop_front()
            .unwrap_or_else(|| self.id_counter.next())
    }

    pub(crate) fn take_ids(&mut self, n: usize) -> id_counter::Take<'_> {
        self.id_counter.take(n)
    }

    pub(crate) fn preload_ids<'a>(&mut self, ids: impl IntoIterator<Item = &'a Id>) {
        self.preloaded_ids.extend(ids)
    }

    pub fn id_path(&self) -> &IdPath {
        &self.id_path
    }
}

impl IdContext for RenderContext {
    fn push_id(&mut self, id: Id) {
        if !id.is_root() {
            self.id_path.push(id);
        }
    }

    fn pop_id(&mut self) {
        self.id_path.pop();
    }
}

#[derive(Debug)]
pub struct MessageContext<T> {
    id_path: IdPathBuf,
    messages: Vec<T>,
}

impl<T> MessageContext<T> {
    pub fn new() -> Self {
        Self {
            id_path: IdPathBuf::new(),
            messages: Vec::new(),
        }
    }

    pub(crate) fn new_sub_context<U>(&self) -> MessageContext<U> {
        MessageContext {
            id_path: self.id_path.clone(),
            messages: Vec::new(),
        }
    }

    pub fn id_path(&self) -> &IdPath {
        &self.id_path
    }

    pub fn push_message(&mut self, message: T) {
        self.messages.push(message);
    }

    pub fn extend_messages(&mut self, messages: impl IntoIterator<Item = T>) {
        self.messages.extend(messages);
    }

    pub fn into_messages(self) -> Vec<T> {
        self.messages
    }
}

impl<T> IdContext for MessageContext<T> {
    fn push_id(&mut self, id: Id) {
        if !id.is_root() {
            self.id_path.push(id);
        }
    }

    fn pop_id(&mut self) {
        self.id_path.pop();
    }
}
