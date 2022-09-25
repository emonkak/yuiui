pub mod id_tree;

pub use id_tree::IdTree;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Id(usize);

impl Id {
    pub const ROOT: Self = Self(0);
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
