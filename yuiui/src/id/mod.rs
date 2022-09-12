pub mod id_tree;

pub use id_tree::IdTree;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Id(usize);

impl Id {
    pub const ROOT: Self = Self(0);

    pub fn from_top(id_path: &IdPath) -> Self {
        id_path.first().copied().unwrap_or(Id::ROOT)
    }

    pub fn from_bottom(id_path: &IdPath) -> Self {
        id_path.last().copied().unwrap_or(Id::ROOT)
    }
}

impl Into<usize> for Id {
    fn into(self) -> usize {
        self.0
    }
}

pub type IdPath = [Id];

pub type IdPathBuf = Vec<Id>;

pub type Depth = usize;

#[derive(Debug)]
pub struct IdCounter {
    count: usize,
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
