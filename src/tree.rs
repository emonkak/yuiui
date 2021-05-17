use std::cell::Cell;
use std::fmt;
use std::ops::Deref;
use std::rc::{Rc, Weak};
use std::ptr;

#[derive(Debug)]
pub struct Tree<T>(Rc<Link<T>>);

pub struct Link<T> {
    current: Node<T>,
    parent: Cell<Weak<Link<T>>>,
    prev_sibling: Cell<Weak<Link<T>>>,
    next_sibling: Cell<Option<Tree<T>>>,
}

pub struct Node<T> {
    data: T,
    first_child: Cell<Option<Tree<T>>>,
    last_child: Cell<Weak<Link<T>>>,
}

pub trait Rearrange<T> {
    fn rearrange(self, new_parent: Weak<Link<T>>, new_prev_sibling: Weak<Link<T>>, new_next_sibling: Option<Tree<T>>) -> Tree<T>;
}

impl<T> Rearrange<T> for Tree<T> {
    fn rearrange(self, new_parent: Weak<Link<T>>, new_prev_sibling: Weak<Link<T>>, new_next_sibling: Option<Tree<T>>) -> Tree<T> {
        self.attach(new_parent, new_prev_sibling, new_next_sibling);
        self
    }
}

impl<T> Rearrange<T> for Node<T> {
    fn rearrange(self, new_parent: Weak<Link<T>>, new_prev_sibling: Weak<Link<T>>, new_next_sibling: Option<Tree<T>>) -> Tree<T> {
        Tree::from(Link {
            current: self,
            parent: Cell::new(new_parent),
            prev_sibling: Cell::new(new_prev_sibling),
            next_sibling: Cell::new(new_next_sibling),
        })
    }
}

impl<T> Tree<T> {
    pub fn append(&self, node: impl Rearrange<T>) -> Tree<T> {
        let last_child_weak = self.current.last_child.take();

        match last_child_weak.upgrade() {
            Some(last_child) => {
                let new_tree = node.rearrange(self.downgrade(), last_child_weak, None);
                self.current.last_child.set(new_tree.downgrade());
                last_child.next_sibling.set(Some(new_tree.clone()));
                new_tree
            }
            None => {
                let new_tree = node.rearrange(self.downgrade(), Weak::new(), None);
                self.current.last_child.set(new_tree.downgrade());
                self.current.first_child.set(Some(new_tree.clone()));
                new_tree
            }
        }
    }

    pub fn prepend(&self, node: impl Rearrange<T>) -> Tree<T> {
        match self.current.first_child.take() {
            Some(first_child) => {
                let new_tree = node.rearrange(self.downgrade(), Weak::new(), Some(first_child.clone()));
                self.current.first_child.set(Some(new_tree.clone()));
                first_child.prev_sibling.set(new_tree.downgrade());
                new_tree
            }
            None => {
                let new_tree = node.rearrange(self.downgrade(), Weak::new(), None);
                self.current.first_child.set(Some(new_tree.clone()));
                self.current.last_child.set(new_tree.downgrade());
                new_tree
            }
        }
    }

    pub fn insert_before(&self, node: impl Rearrange<T>) -> Tree<T> {
        let prev_sibling_weak = self.prev_sibling.take();

        let new_tree = match prev_sibling_weak.upgrade() {
            Some(prev_sibling) => {
                let new_tree = node.rearrange(
                    clone_cell_inner(&self.parent),
                    prev_sibling_weak,
                    Some(self.clone())
                );
                prev_sibling.next_sibling.set(Some(new_tree.clone()));
                new_tree
            }
            None => {
                let parent = self.parent().expect("Only one element on root allowed.");
                let new_tree = node.rearrange(
                    parent.downgrade(),
                    Weak::new(),
                    Some(self.clone())
                );
                parent.current.first_child.set(Some(new_tree.clone()));
                new_tree
            }
        };

        self.prev_sibling.set(new_tree.downgrade());

        new_tree
    }

    pub fn insert_after(&self, node: impl Rearrange<T>) -> Tree<T> {
        let new_tree = match self.next_sibling.take() {
            Some(next_sibling) => {
                let new_tree = node.rearrange(
                    clone_cell_inner(&self.parent),
                    self.downgrade(),
                    Some(next_sibling.clone())
                );
                next_sibling.prev_sibling.set(new_tree.downgrade());
                new_tree
            }
            None => {
                let parent = self.parent().expect("Only one element on root allowed.");
                let new_tree = node.rearrange(
                    parent.downgrade(),
                    self.downgrade(),
                    None
                );
                parent.current.last_child.set(new_tree.downgrade());
                new_tree
            }
        };

        self.next_sibling.set(Some(new_tree.clone()));

        new_tree
    }

    fn downgrade(&self) -> Weak<Link<T>> {
        Rc::downgrade(&self.0)
    }
}

impl<T> From<Link<T>> for Tree<T> {
    fn from(tree: Link<T>) -> Tree<T> {
        Tree(Rc::new(tree))
    }
}

impl<T> From<Node<T>> for Tree<T> {
    fn from(node: Node<T>) -> Tree<T> {
        Tree::from(Link {
            current: node,
            parent: Cell::new(Weak::new()),
            next_sibling: Cell::new(None),
            prev_sibling: Cell::new(Weak::new()),
        })
    }
}

impl<T> Clone for Tree<T> {
    fn clone(&self) -> Tree<T> {
        Tree(Rc::clone(&self.0))
    }
}

impl<T> Deref for Tree<T> {
    type Target = Link<T>;

    fn deref(&self) -> &Link<T> {
        &self.0
    }
}

impl<T> Link<T> {
    pub fn detach(&self) {
        self.attach(Weak::new(), Weak::new(), None);
    }

    pub fn replace(&self, node: impl Rearrange<T>) -> Tree<T> {
        let new_tree = node.rearrange(
            self.parent.take(),
            self.prev_sibling.take(),
            self.next_sibling.take(),
        );

        match (new_tree.prev_sibling(), new_tree.next_sibling()) {
            (Some(prev_sibling), Some(next_sibling)) => {
                prev_sibling.next_sibling.set(Some(new_tree.clone()));
                next_sibling.prev_sibling.set(new_tree.downgrade());
            }
            (Some(prev_sibling), None) => {
                prev_sibling.next_sibling.set(Some(new_tree.clone()));

                if let Some(parent) = new_tree.parent() {
                    parent.current.last_child.set(new_tree.downgrade());
                }
            }
            (None, Some(next_sibling)) => {
                next_sibling.prev_sibling.set(new_tree.downgrade());

                if let Some(parent) = new_tree.parent() {
                    parent.current.first_child.set(Some(new_tree.clone()));
                }
            }
            (None, None) => {
                if let Some(parent) = new_tree.parent() {
                    parent.current.first_child.set(Some(new_tree.clone()));
                    parent.current.last_child.set(new_tree.downgrade());
                }
            }
        };

        new_tree
    }

    pub fn data(&self) -> &T {
        &self.current.data
    }

    pub fn first_child(&self) -> Option<Tree<T>> {
        clone_cell_inner(&self.current.first_child)
    }

    pub fn last_child(&self) -> Option<Tree<T>> {
        clone_cell_inner(&self.current.last_child).upgrade().map(Tree)
    }

    pub fn parent(&self) -> Option<Tree<T>> {
        clone_cell_inner(&self.parent).upgrade().map(Tree)
    }

    pub fn children(&self) -> impl DoubleEndedIterator<Item = Tree<T>> {
        Siblings {
            next: self.first_child(),
        }
    }

    pub fn children_rev(&self) -> impl DoubleEndedIterator<Item = Tree<T>> {
        Siblings {
            next: self.last_child(),
        }.rev()
    }

    pub fn descendants(&self) -> impl Iterator<Item = Tree<T>> + '_ {
        Descendants {
            next: self.first_child(),
            root: &self.current,
        }
    }

    pub fn next_sibling(&self) -> Option<Tree<T>> {
        clone_cell_inner(&self.next_sibling)
    }

    pub fn prev_sibling(&self) -> Option<Tree<T>> {
        clone_cell_inner(&self.prev_sibling).upgrade().map(Tree)
    }

    pub fn ancestors(&self) -> impl Iterator<Item = Tree<T>> {
        Ancestors {
            next: self.parent(),
        }
    }

    pub fn next_siblings(&self) -> impl DoubleEndedIterator<Item = Tree<T>> {
        Siblings {
            next: self.next_sibling(),
        }
    }

    pub fn prev_siblings(&self) -> impl DoubleEndedIterator<Item = Tree<T>> {
        Siblings {
            next: self.prev_sibling()
        }.rev()
    }

    fn attach(&self, new_parent: Weak<Link<T>>, new_prev_sibling: Weak<Link<T>>, new_next_sibling: Option<Tree<T>>) {
        let prev_sibling_weak = self.prev_sibling.replace(new_prev_sibling);

        match (
            prev_sibling_weak.upgrade(),
            self.next_sibling.replace(new_next_sibling)
        ) {
            (Some(prev_sibling), Some(next_sibling)) => {
                next_sibling.prev_sibling.set(prev_sibling_weak);
                prev_sibling.next_sibling.set(Some(next_sibling));
                self.parent.set(new_parent);
            }
            (Some(prev_sibling), None) => {
                prev_sibling.next_sibling.set(None);

                if let Some(parent) = self.parent.replace(new_parent).upgrade() {
                    parent.current.last_child.set(prev_sibling_weak);
                }
            }
            (None, Some(next_sibling)) => {
                next_sibling.prev_sibling.set(Weak::new());

                if let Some(parent) = self.parent.replace(new_parent).upgrade() {
                    parent.current.first_child.set(Some(next_sibling));
                }
            }
            (None, None) => {
                if let Some(parent) = self.parent.replace(new_parent).upgrade() {
                    parent.current.first_child.set(None);
                    parent.current.last_child.set(Weak::new());
                }
            }
        };
    }
}

impl<T: fmt::Debug> ToString for Link<T> {
    fn to_string(&self) -> String {
        fn to_string_link<T>(link: &Link<T>, level: usize) -> String where T: fmt::Debug {
            let current_string = to_string_node(&link.current, level);
            if level > 0 {
                link.next_sibling()
                    .map(|next_sibling| {
                        format!("{}\n{}", current_string, to_string_link(&next_sibling, level))
                    })
                    .unwrap_or(current_string)
            } else {
                current_string
            }
        }

        fn to_string_node<T>(node: &Node<T>, level: usize) -> String where T: fmt::Debug {
            let indent_string = unsafe { String::from_utf8_unchecked(vec![b'\t'; level]) };
            let children_string = clone_cell_inner(&node.first_child)
                .map(|first_child| {
                    format!("\n{}\n{}", to_string_link(&first_child, level + 1), indent_string)
                })
                .unwrap_or_default();
            format!(
                "{}<{:?} data=\"{}\">{}</{:?}>",
                indent_string,
                node as *const _,
                format!("{:?}", node.data).trim_matches('"'),
                children_string,
                node as *const _,
            )
        }

        to_string_link(&self, 0)
    }
}

impl<T: fmt::Debug> fmt::Debug for Link<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("Link")
            .field("current", &self.current)
            .field("parent", &clone_cell_inner(&self.parent))
            .field("next_sibling", &clone_cell_inner(&self.next_sibling))
            .field("prev_sibling", &clone_cell_inner(&self.prev_sibling))
            .finish()
    }
}

impl<T> Node<T> {
    pub fn new(data: T) -> Node<T> {
        Node {
            data,
            first_child: Cell::new(None),
            last_child: Cell::new(Weak::new()),
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for Node<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("Node")
            .field("data", &self.data)
            .field("first_child", &clone_cell_inner(&self.first_child))
            .field("last_child", &clone_cell_inner(&self.last_child))
            .finish()
    }
}

struct Siblings<T> {
    next: Option<Tree<T>>,
}

impl<T> Iterator for Siblings<T> {
    type Item = Tree<T>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next.take() {
            Some(current) => {
                self.next = current.next_sibling();
                Some(current)
            },
            None => None,
        }
    }
}

impl<T> DoubleEndedIterator for Siblings<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        match self.next.take() {
            Some(current) => {
                self.next = current.prev_sibling();
                Some(current)
            },
            None => None,
        }
    }
}

pub struct Descendants<'a, T> {
    next: Option<Tree<T>>,
    root: &'a Node<T>,
}

impl<'a, T> Iterator for Descendants<'a, T> {
    type Item = Tree<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.take().map(|current| {
            self.next = current.first_child()
                .or_else(|| current.next_sibling())
                .or_else(|| {
                    let mut current = current.parent();
                    loop {
                        match current {
                            Some(parent) if !ptr::eq(&parent.current, self.root) => {
                                if let Some(next_sibling) = parent.next_sibling() {
                                    return Some(next_sibling);
                                }
                                current = parent.parent();
                            }
                            _ => break
                        }
                    }
                    None
                });
            current
        })
    }
}

struct Ancestors<T> {
    next: Option<Tree<T>>,
}

impl<T> Iterator for Ancestors<T> {
    type Item = Tree<T>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next.take() {
            Some(current) => {
                self.next = current.parent();
                Some(current)
            },
            None => None,
        }
    }
}

fn clone_cell_inner<T: Clone>(cell: &Cell<T>) -> T {
    unsafe { &*cell.as_ptr() }.clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    impl<T> PartialEq for Tree<T> {
        fn eq(&self, other: &Self) -> bool {
            Rc::ptr_eq(&self.0, &other.0)
        }
    }

    impl<T> Eq for Tree<T> {
    }

    #[test]
    fn test_append() {
        let root = Tree::from(Node::new("root"));

        assert_tree(&root, &"root", None, None, None, None, None);

        let foo = root.append(Node::new("foo"));

        assert_tree(&root, &"root", Some(&foo), Some(&foo), None, None, None);
        assert_tree(&foo, &"foo", None, None, Some(&root), None, None);

        let bar = root.append(Node::new("bar"));

        assert_tree(&root, &"root", Some(&foo), Some(&bar), None, None, None);
        assert_tree(&foo, &"foo", None, None, Some(&root), None, Some(&bar));
        assert_tree(&bar, &"bar", None, None, Some(&root), Some(&foo), None);
    }

    #[test]
    fn test_prepend() {
        let root = Tree::from(Node::new("root"));

        assert_tree(&root, &"root", None, None, None, None, None);

        let foo = root.prepend(Node::new("foo"));

        assert_tree(&root, &"root", Some(&foo), Some(&foo), None, None, None);
        assert_tree(&foo, &"foo", None, None, Some(&root), None, None);

        let bar = root.prepend(Node::new("bar"));

        assert_tree(&root, &"root", Some(&bar), Some(&foo), None, None, None);
        assert_tree(&foo, &"foo", None, None, Some(&root), Some(&bar), None);
        assert_tree(&bar, &"bar", None, None, Some(&root), None, Some(&foo));
    }

    #[test]
    fn test_insert_before() {
        let root = Tree::from(Node::new("root"));
        let foo = root.append(Node::new("foo"));
        let bar = root.append(Node::new("bar"));
        let baz = foo.insert_before(Node::new("baz"));
        let qux = foo.insert_before(Node::new("qux"));

        assert_tree(&root, &"root", Some(&baz), Some(&bar), None, None, None);
        assert_tree(&foo, &"foo", None, None, Some(&root), Some(&qux), Some(&bar));
        assert_tree(&bar, &"bar", None, None, Some(&root), Some(&foo), None);
        assert_tree(&baz, &"baz", None, None, Some(&root), None, Some(&qux));
        assert_tree(&qux, &"qux", None, None, Some(&root), Some(&baz), Some(&foo));
    }

    #[should_panic]
    #[test]
    fn test_insert_before_should_panic() {
        let root = Tree::from(Node::new("root"));
        root.insert_before(Node::new("foo"));
    }

    #[test]
    fn test_insert_after() {
        let root = Tree::from(Node::new("root"));
        let foo = root.append(Node::new("foo"));
        let bar = root.append(Node::new("bar"));
        let baz = bar.insert_after(Node::new("baz"));
        let qux = bar.insert_after(Node::new("qux"));

        assert_tree(&root, &"root", Some(&foo), Some(&baz), None, None, None);
        assert_tree(&foo, &"foo", None, None, Some(&root), None, Some(&bar));
        assert_tree(&bar, &"bar", None, None, Some(&root), Some(&foo), Some(&qux));
        assert_tree(&baz, &"baz", None, None, Some(&root), Some(&qux), None);
        assert_tree(&qux, &"qux", None, None, Some(&root), Some(&bar), Some(&baz));
    }

    #[should_panic]
    #[test]
    fn test_insert_after_should_panic() {
        let root = Tree::from(Node::new("root"));
        root.insert_after(Node::new("foo"));
    }

    #[test]
    fn test_detach() {
        let root = Tree::from(Node::new("root"));
        let foo = root.append(Node::new("foo"));
        let bar = root.append(Node::new("bar"));
        let baz = root.append(Node::new("baz"));
        let qux = root.append(Node::new("qux"));

        bar.detach();

        assert_tree(&root, &"root", Some(&foo), Some(&qux), None, None, None);
        assert_tree(&foo, &"foo", None, None, Some(&root), None, Some(&baz));
        assert_tree(&bar, &"bar", None, None, None, None, None);
        assert_tree(&baz, &"baz", None, None, Some(&root), Some(&foo), Some(&qux));
        assert_tree(&qux, &"qux", None, None, Some(&root), Some(&baz), None);

        qux.detach();

        assert_tree(&root, &"root", Some(&foo), Some(&baz), None, None, None);
        assert_tree(&foo, &"foo", None, None, Some(&root), None, Some(&baz));
        assert_tree(&bar, &"bar", None, None, None, None, None);
        assert_tree(&baz, &"baz", None, None, Some(&root), Some(&foo), None);
        assert_tree(&qux, &"qux", None, None, None, None, None);

        foo.detach();

        assert_tree(&root, &"root", Some(&baz), Some(&baz), None, None, None);
        assert_tree(&foo, &"foo", None, None, None, None, None);
        assert_tree(&bar, &"bar", None, None, None, None, None);
        assert_tree(&baz, &"baz", None, None, Some(&root), None, None);
        assert_tree(&qux, &"qux", None, None, None, None, None);

        baz.detach();

        assert_tree(&root, &"root", None, None, None, None, None);
        assert_tree(&foo, &"foo", None, None, None, None, None);
        assert_tree(&bar, &"bar", None, None, None, None, None);
        assert_tree(&baz, &"baz", None, None, None, None, None);
        assert_tree(&qux, &"qux", None, None, None, None, None);
    }

    #[test]
    fn test_replace() {
        let root = Tree::from(Node::new("root"));
        let foo = root.append(Node::new("foo"));
        let bar = root.append(Node::new("bar"));
        let baz = root.append(Node::new("baz"));

        let new_bar = bar.replace(Node::new("new_bar"));

        assert_tree(&root, &"root", Some(&foo), Some(&baz), None, None, None);
        assert_tree(&foo, &"foo", None, None, Some(&root), None, Some(&new_bar));
        assert_tree(&bar, &"bar", None, None, None, None, None);
        assert_tree(&baz, &"baz", None, None, Some(&root), Some(&new_bar), None);

        assert_tree(&new_bar, &"new_bar", None, None, Some(&root), Some(&foo), Some(&baz));

        let new_baz = baz.replace(Node::new("new_baz"));

        assert_tree(&root, &"root", Some(&foo), Some(&new_baz), None, None, None);
        assert_tree(&foo, &"foo", None, None, Some(&root), None, Some(&new_bar));
        assert_tree(&bar, &"bar", None, None, None, None, None);
        assert_tree(&baz, &"baz", None, None, None, None, None);

        assert_tree(&new_bar, &"new_bar", None, None, Some(&root), Some(&foo), Some(&new_baz));
        assert_tree(&new_baz, &"new_baz", None, None, Some(&root), Some(&new_bar), None);

        let new_foo = foo.replace(Node::new("new_foo"));

        assert_tree(&root, &"root", Some(&new_foo), Some(&new_baz), None, None, None);
        assert_tree(&foo, &"foo", None, None, None, None, None);
        assert_tree(&bar, &"bar", None, None, None, None, None);
        assert_tree(&baz, &"baz", None, None, None, None, None);

        assert_tree(&new_foo, &"new_foo", None, None, Some(&root), None, Some(&new_bar));
        assert_tree(&new_bar, &"new_bar", None, None, Some(&root), Some(&new_foo), Some(&new_baz));
        assert_tree(&new_baz, &"new_baz", None, None, Some(&root), Some(&new_bar), None);
    }

    #[test]
    fn test_replace_only_child() {
        let root = Tree::from(Node::new("root"));
        let foo = root.append(Node::new("foo"));
        let new_foo = foo.replace(Node::new("new_foo"));

        assert_tree(&root, &"root", Some(&new_foo), Some(&new_foo), None, None, None);
        assert_tree(&foo, &"foo", None, None, None, None, None);
        assert_tree(&new_foo, &"new_foo", None, None, Some(&root), None, None);
    }

    #[test]
    fn test_children() {
        let root = Tree::from(Node::new("root"));
        let foo = root.append(Node::new("foo"));
        let bar = root.append(Node::new("bar"));
        let baz = foo.append(Node::new("baz"));
        let qux = baz.append(Node::new("qux"));
        let quux = qux.append(Node::new("quux"));
        let corge = baz.append(Node::new("corge"));

        assert_iterator(root.children(), &[&foo, &bar]);

        assert_iterator(foo.children(), &[&baz]);
        assert_iterator(bar.children(), &[]);
        assert_iterator(baz.children(), &[&qux, &corge]);
        assert_iterator(qux.children(), &[&quux]);
        assert_iterator(quux.children(), &[]);
        assert_iterator(corge.children(), &[]);

        assert_iterator(root.children_rev(), &[&bar, &foo]);
        assert_iterator(foo.children_rev(), &[&baz]);
        assert_iterator(bar.children_rev(), &[]);
        assert_iterator(baz.children_rev(), &[&corge, &qux]);
        assert_iterator(qux.children_rev(), &[&quux]);
        assert_iterator(quux.children_rev(), &[]);
        assert_iterator(corge.children_rev(), &[]);
    }

    #[test]
    fn test_descendants() {
        let root = Tree::from(Node::new("root"));
        let foo = root.append(Node::new("foo"));
        let bar = root.append(Node::new("bar"));
        let baz = foo.append(Node::new("baz"));
        let qux = baz.append(Node::new("qux"));
        let quux = qux.append(Node::new("quux"));
        let corge = baz.append(Node::new("corge"));

        assert_iterator(root.descendants(), &[&foo, &baz, &qux, &quux, &corge, &bar]);
        assert_iterator(foo.descendants(), &[&baz, &qux, &quux, &corge]);
        assert_iterator(bar.descendants(), &[]);
        assert_iterator(baz.descendants(), &[&qux, &quux, &corge]);
        assert_iterator(qux.descendants(), &[&quux]);
        assert_iterator(quux.descendants(), &[]);
        assert_iterator(corge.descendants(), &[]);
    }

    #[test]
    fn test_parents() {
        let root = Tree::from(Node::new("root"));
        let foo = root.append(Node::new("foo"));
        let bar = root.append(Node::new("bar"));
        let baz = foo.append(Node::new("baz"));
        let qux = baz.append(Node::new("qux"));
        let quux = qux.append(Node::new("quux"));
        let corge = baz.append(Node::new("corge"));

        assert_iterator(root.ancestors(), &[]);
        assert_iterator(foo.ancestors(), &[&root]);
        assert_iterator(bar.ancestors(), &[&root]);
        assert_iterator(baz.ancestors(), &[&foo, &root]);
        assert_iterator(qux.ancestors(), &[&baz, &foo, &root]);
        assert_iterator(quux.ancestors(), &[&qux, &baz, &foo, &root]);
        assert_iterator(corge.ancestors(), &[&baz, &foo, &root]);
    }

    #[test]
    fn test_siblings() {
        let root = Tree::from(Node::new("root"));
        let foo = root.append(Node::new("foo"));
        let bar = root.append(Node::new("bar"));
        let baz = root.append(Node::new("baz"));
        let qux = root.append(Node::new("qux"));
        let quux = root.append(Node::new("quux"));
        let corge = root.append(Node::new("corge"));

        assert_iterator(root.prev_siblings(), &[]);
        assert_iterator(foo.prev_siblings(), &[]);
        assert_iterator(bar.prev_siblings(), &[&foo]);
        assert_iterator(baz.prev_siblings(), &[&bar, &foo]);
        assert_iterator(qux.prev_siblings(), &[&baz, &bar, &foo]);
        assert_iterator(quux.prev_siblings(), &[&qux, &baz, &bar, &foo]);
        assert_iterator(corge.prev_siblings(), &[&quux, &qux, &baz, &bar, &foo]);

        assert_iterator(root.next_siblings(), &[]);
        assert_iterator(foo.next_siblings(), &[&bar, &baz, &qux, &quux, &corge]);
        assert_iterator(bar.next_siblings(), &[&baz, &qux, &quux, &corge]);
        assert_iterator(baz.next_siblings(), &[&qux, &quux, &corge]);
        assert_iterator(qux.next_siblings(), &[&quux, &corge]);
        assert_iterator(quux.next_siblings(), &[&corge]);
        assert_iterator(corge.next_siblings(), &[]);
    }

    fn assert_tree<T: PartialEq + fmt::Debug>(
        tree: &Tree<T>,
        expeted_data: &T,
        expeted_first_child: Option<&Tree<T>>,
        expeted_last_child: Option<&Tree<T>>,
        expeted_parent: Option<&Tree<T>>,
        expeted_prev_sibling: Option<&Tree<T>>,
        expeted_next_sibling: Option<&Tree<T>>
    ) {
        assert_eq!(tree.data(), expeted_data, "data");
        assert_eq!(tree.first_child().as_ref(), expeted_first_child, "first_child");
        assert_eq!(tree.last_child().as_ref(), expeted_last_child, "last_child");
        assert_eq!(tree.parent().as_ref(), expeted_parent, "parent");
        assert_eq!(tree.prev_sibling().as_ref(), expeted_prev_sibling, "prev_sibling");
        assert_eq!(tree.next_sibling().as_ref(), expeted_next_sibling, "next_sibling");
    }

    fn assert_iterator<T: PartialEq + fmt::Debug>(iter: impl Iterator<Item = T>, expeted_data: &[&T]) {
        assert_eq!(&to_vec_ref(&iter.collect()), expeted_data);
    }

    fn to_vec_ref<T>(xs: &Vec<T>) -> Vec<&T> {
        let mut ys = Vec::with_capacity(xs.len());

        for x in xs {
            ys.push(x);
        }

        ys
    }
}
