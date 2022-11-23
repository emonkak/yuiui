use super::{Depth, Id, IdPath, IdPathBuf};
use crate::state::Atom;

#[derive(Debug)]
pub struct IdContext {
    id_path: IdPathBuf,
    depth: Depth,
    counter: usize,
}

impl IdContext {
    pub fn new() -> Self {
        Self {
            id_path: IdPathBuf::new(),
            depth: 0,
            counter: 1,
        }
    }

    pub fn use_atom<'a, T>(&self, atom: &'a Atom<T>) -> &'a T {
        atom.subscribe(&self.id_path, self.depth);
        atom.value()
    }

    pub fn id_path(&self) -> &IdPath {
        &self.id_path
    }

    pub fn depth(&self) -> Depth {
        self.depth
    }

    pub(crate) fn next_id(&mut self) -> Id {
        let id = self.counter;
        self.counter += 1;
        Id::new(id)
    }

    pub(crate) fn push_id(&mut self, id: Id) {
        if !id.is_root() {
            self.id_path.push(id);
        }
    }

    pub(crate) fn pop_id(&mut self) {
        self.id_path.pop();
    }

    pub(crate) fn set_depth(&mut self, depth: Depth) {
        self.depth = depth;
    }
}
