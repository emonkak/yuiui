use std::fmt;

use crate::hlist::{HCons, HList, HNil};

trait DebugHList: HList {
    fn fmt(&self, debug_list: &mut fmt::DebugList) -> fmt::Result;
}

impl DebugHList for HNil {
    fn fmt(&self, debug_list: &mut fmt::DebugList) -> fmt::Result {
        debug_list.finish()
    }
}

impl fmt::Debug for HNil {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("hlist![]")
    }
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

impl<Head, Tail> fmt::Debug for HCons<Head, Tail>
where
    Head: fmt::Debug,
    Tail: DebugHList,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("hlist!")?;
        DebugHList::fmt(self, &mut f.debug_list())
    }
}
