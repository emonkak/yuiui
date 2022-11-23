use super::{Id, IdPath, IdPathBuf, Level};

#[derive(Debug)]
pub struct IdStack {
    id_path: IdPathBuf,
    level: Level,
    next_id: Id,
}

impl IdStack {
    pub fn new() -> Self {
        Self {
            id_path: IdPathBuf::new(),
            level: 0,
            next_id: Id::ROOT.next(),
        }
    }

    pub fn id(&self) -> Id {
        Id::from(self.id_path.as_slice())
    }

    pub fn id_path(&self) -> &IdPath {
        &self.id_path
    }

    pub fn level(&self) -> Level {
        self.level
    }

    pub fn push(&mut self, id: Id) {
        assert!(!id.is_root());
        self.id_path.push(id);
    }

    pub fn pop(&mut self) {
        assert!(!self.id_path.is_empty());
        self.id_path.pop();
    }

    pub fn next(&mut self) -> Id {
        let id = self.next_id;
        self.next_id = id.next();
        id
    }

    pub fn set_level(&mut self, level: Level) {
        self.level = level;
    }
}
