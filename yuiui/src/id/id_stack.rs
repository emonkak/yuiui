use super::{Id, IdPath, IdPathBuf};

#[derive(Debug)]
pub struct IdStack {
    id_path: IdPathBuf,
    counter: usize,
}

impl IdStack {
    pub fn new() -> Self {
        Self {
            id_path: IdPathBuf::new(),
            counter: 1,
        }
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

    pub fn id_path(&self) -> &IdPath {
        &self.id_path
    }
}
