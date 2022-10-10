use std::collections::VecDeque;

use super::{Id, IdPath, IdPathBuf};

#[derive(Debug)]
pub struct IdContext {
    id_path: IdPathBuf,
    counter: usize,
    preloaded_ids: VecDeque<Id>,
}

impl IdContext {
    pub fn new() -> Self {
        Self {
            id_path: IdPathBuf::new(),
            counter: 1,
            preloaded_ids: VecDeque::new(),
        }
    }

    pub(crate) fn next_id(&mut self) -> Id {
        self.preloaded_ids.pop_front().unwrap_or_else(|| {
            let id = self.counter;
            self.counter += 1;
            Id::new(id)
        })
    }

    pub(crate) fn take_ids(&mut self, n: usize) -> Vec<Id> {
        let mut ids = Vec::with_capacity(n);
        for _ in 0..n {
            ids.push(self.next_id());
        }
        ids
    }

    pub(crate) fn preload_ids<'a>(&mut self, ids: impl IntoIterator<Item = &'a Id>) {
        self.preloaded_ids.extend(ids)
    }

    pub(crate) fn push_id(&mut self, id: Id) {
        if !id.is_root() {
            self.id_path.push(id);
        }
    }

    pub(crate) fn pop_id(&mut self) {
        self.id_path.pop();
    }

    pub fn id_path(&self) -> &IdPath {
        &self.id_path
    }
}
