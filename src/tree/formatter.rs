use std::fmt;

use super::{NodeId, Tree};

pub struct TreeFormatter<'a, T, FOpen, FClose> {
    pub(super) tree: &'a Tree<T>,
    pub(super) node_id: NodeId,
    pub(super) format_open: FOpen,
    pub(super) format_close: FClose
}

impl<'a, T, FOpen, FClose> TreeFormatter<'a, T, FOpen, FClose>
where
    T: fmt::Display,
    FOpen: Fn(&mut fmt::Formatter, NodeId, &T) -> fmt::Result,
    FClose: Fn(&mut fmt::Formatter, NodeId, &T) -> fmt::Result {
    fn fmt_rec(
        &self,
        f: &mut fmt::Formatter,
        node_id: NodeId,
        level: usize
    ) -> fmt::Result where T: fmt::Display {
        let indent_str = unsafe { String::from_utf8_unchecked(vec![b'\t'; level]) };
        let link = &self.tree.arena[node_id];

        write!(f, "{}", indent_str)?;

        (self.format_open)(f, node_id, &link.current.data)?;

        if let Some(child_id) = link.current.first_child {
            write!(f, "\n")?;
            self.fmt_rec(f, child_id, level + 1)?;
            write!(f, "\n{}", indent_str)?;
        }

        (self.format_close)(f, node_id, &link.current.data)?;

        if let Some(child_id) = link.next_sibling {
            write!(f, "\n")?;
            self.fmt_rec(f, child_id, level)?;
        }

        Ok(())
    }
}

impl<'a, T, FOpen, FClose> fmt::Display for TreeFormatter<'a, T, FOpen, FClose>
where
    T: fmt::Display,
    FOpen: Fn(&mut fmt::Formatter, NodeId, &T) -> fmt::Result,
    FClose: Fn(&mut fmt::Formatter, NodeId, &T) -> fmt::Result {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_rec(f, self.node_id, 0)
    }
}
