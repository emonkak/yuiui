pub mod id_tree;

mod id_stack;

pub use id_stack::IdStack;
pub use id_tree::IdTree;

use std::num::NonZeroU32;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Id(NonZeroU32);

impl Id {
    pub const ROOT: Self = Self(unsafe { NonZeroU32::new_unchecked(1) });

    const fn new(id: u32) -> Self {
        assert!(id > 0);
        Self(unsafe { NonZeroU32::new_unchecked(id) })
    }

    pub fn is_root(&self) -> bool {
        self == &Self::ROOT
    }

    const fn next(&self) -> Self {
        Self::new(self.0.get() + 1)
    }
}

impl<'a> From<&'a IdPath> for Id {
    fn from(id_path: &'a IdPath) -> Self {
        id_path.last().copied().unwrap_or(Id::ROOT)
    }
}

impl Into<NonZeroU32> for Id {
    fn into(self) -> NonZeroU32 {
        self.0
    }
}

impl Into<u32> for Id {
    fn into(self) -> u32 {
        self.0.get()
    }
}

pub type IdPath = [Id];

pub type IdPathBuf = Vec<Id>;

pub type Level = u32;

#[derive(Clone, Debug)]
pub struct Subscriber {
    pub(crate) id_path: IdPathBuf,
    pub(crate) level: Level,
}
