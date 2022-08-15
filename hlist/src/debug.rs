use std::fmt;

use super::{HCons, HList, HNil};

pub trait DebugHList: HList {
    fn fmt(&self, debug_list: &mut fmt::DebugList) -> fmt::Result;
}

impl<Head, Tail> DebugHList for HCons<Head, Tail>
where
    Head: fmt::Debug,
    Tail: DebugHList,
{
    fn fmt(&self, debug_list: &mut fmt::DebugList) -> fmt::Result {
        debug_list.entry(&self.head);
        self.tail.fmt(debug_list)
    }
}

impl DebugHList for HNil {
    fn fmt(&self, debug_list: &mut fmt::DebugList) -> fmt::Result {
        debug_list.finish()
    }
}
