mod id_tree;

pub use id_tree::{Cursor, IdTree, Node};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Id(u64);

impl Id {
    pub const ROOT: Self = Self(0);

    pub fn from_top(id_path: &IdPath) -> Self {
        id_path.first().copied().unwrap_or(Id::ROOT)
    }

    pub fn from_bottom(id_path: &IdPath) -> Self {
        id_path.last().copied().unwrap_or(Id::ROOT)
    }
}

pub type IdPath = [Id];

pub type IdPathBuf = Vec<Id>;

pub type Depth = usize;

#[derive(Debug)]
pub struct IdCounter {
    count: u64,
}

impl IdCounter {
    pub fn new() -> Self {
        Self { count: 0 }
    }

    pub fn next(&mut self) -> Id {
        let id = self.count;
        self.count += 1;
        Id(id)
    }
}
