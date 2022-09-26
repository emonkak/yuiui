pub mod id_tree;

pub use id_tree::IdTree;
pub use std::num::NonZeroUsize;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Id(NonZeroUsize);

impl Id {
    pub const ROOT: Self = Self(unsafe { NonZeroUsize::new_unchecked(1) });

    fn new(id: usize) -> Self {
        assert!(id > 0);
        Self(unsafe { NonZeroUsize::new_unchecked(id) })
    }
}

impl Into<NonZeroUsize> for Id {
    fn into(self) -> NonZeroUsize {
        self.0
    }
}

impl Into<usize> for Id {
    fn into(self) -> usize {
        self.0.get()
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
        Self { count: 1 }
    }

    pub fn next(&mut self) -> Id {
        let id = self.count;
        self.count += 1;
        Id::new(id)
    }
}
