pub mod id_tree;

mod id_context;

pub use id_context::IdContext;
pub use id_tree::IdTree;

use std::num::NonZeroUsize;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Id(NonZeroUsize);

impl Id {
    pub const ROOT: Self = Self(unsafe { NonZeroUsize::new_unchecked(1) });

    fn new(id: usize) -> Self {
        assert!(id > 0);
        Self(unsafe { NonZeroUsize::new_unchecked(id) })
    }

    pub fn is_root(&self) -> bool {
        self == &Self::ROOT
    }
}

impl<'a> From<&'a IdPath> for Id {
    fn from(id_path: &'a IdPath) -> Self {
        id_path.last().copied().unwrap_or(Id::ROOT)
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
