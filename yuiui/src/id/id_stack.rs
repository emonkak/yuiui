use super::{Depth, Id, IdPath, IdPathBuf};

#[derive(Debug)]
pub struct IdStack {
    id_path: IdPathBuf,
    depth: Depth,
    counter: usize,
}

impl IdStack {
    pub fn new() -> Self {
        Self {
            id_path: IdPathBuf::new(),
            depth: 0,
            counter: 1,
        }
    }

    pub fn id_path(&self) -> &IdPath {
        &self.id_path
    }

    pub fn depth(&self) -> Depth {
        self.depth
    }

    pub fn next_id(&mut self) -> Id {
        let id = self.counter;
        self.counter += 1;
        Id::new(id)
    }

    pub fn push_id(&mut self, id: Id) {
        if !id.is_root() {
            self.id_path.push(id);
        }
    }

    pub fn pop_id(&mut self) {
        self.id_path.pop();
    }

    pub fn set_depth(&mut self, depth: Depth) {
        self.depth = depth;
    }
}
