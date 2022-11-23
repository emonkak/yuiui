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

    pub fn id(&self) -> Id {
        Id::from(self.id_path.as_slice())
    }

    pub fn id_path(&self) -> &IdPath {
        &self.id_path
    }

    pub fn depth(&self) -> Depth {
        self.depth
    }

    pub fn next(&mut self) -> Id {
        self.counter += 1;
        Id::new(self.counter)
    }

    pub fn push(&mut self, id: Id) {
        if !id.is_root() {
            self.id_path.push(id);
        }
    }

    pub fn pop(&mut self) {
        self.id_path.pop();
    }

    pub fn set_depth(&mut self, depth: Depth) {
        self.depth = depth;
    }
}
