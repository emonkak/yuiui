use std::cell::{Ref, RefCell, RefMut};
use std::convert::TryInto;
use std::fmt::Debug;
use std::rc::{Rc, Weak};

#[derive(Debug)]
pub struct Tree<T>(NodeRef<T>);

#[derive(Debug)]
pub struct Node<T> {
    data: T,
    first_child: Option<NodeRef<T>>,
    last_child: NodeWeak<T>,
    prev_sibling: NodeWeak<T>,
    next_sibling: Option<NodeRef<T>>,
    parent: NodeWeak<T>,
}

type NodeRef<T> = Rc<RefCell<Node<T>>>;
type NodeWeak<T> = Weak<RefCell<Node<T>>>;

impl<T> Tree<T> {
    pub fn append(&self, mut new_node: Node<T>) -> Tree<T> {
        assert!(new_node.prev_sibling.upgrade().is_none());
        assert!(new_node.next_sibling.is_none());
        assert!(new_node.parent.upgrade().is_none());

        new_node.parent = Rc::downgrade(&self.0);

        let mut current = self.0.borrow_mut();

        if let Some(last_child_ref) = current.last_child.upgrade() {
            new_node.prev_sibling = Rc::downgrade(&last_child_ref);
            let new_child_ref = new_node.into_ref();
            current.last_child = Rc::downgrade(&new_child_ref);
            last_child_ref.borrow_mut().next_sibling = Some(Rc::clone(&new_child_ref));
            Tree(new_child_ref)
        } else {
            let new_child_ref = new_node.into_ref();
            current.last_child = Rc::downgrade(&new_child_ref);
            current.first_child = Some(Rc::clone(&new_child_ref));
            Tree(new_child_ref)
        }
    }

    pub fn prepend(&self, mut new_node: Node<T>) -> Tree<T> {
        assert!(new_node.prev_sibling.upgrade().is_none());
        assert!(new_node.next_sibling.is_none());
        assert!(new_node.parent.upgrade().is_none());

        new_node.parent = Rc::downgrade(&self.0);

        let mut current = self.0.borrow_mut();

        if let Some(first_child_ref) = current.first_child.as_ref() {
            new_node.next_sibling = Some(Rc::clone(&first_child_ref));

            let new_child_ref = new_node.into_ref();
            first_child_ref.borrow_mut().prev_sibling = Rc::downgrade(&new_child_ref);
            current.first_child = Some(Rc::clone(&new_child_ref));
            Tree(new_child_ref)
        } else {
            let new_child_ref = new_node.into_ref();
            current.last_child = Rc::downgrade(&new_child_ref);
            current.first_child = Some(Rc::clone(&new_child_ref));
            Tree(new_child_ref)
        }
    }

    pub fn insert_before(&self, mut new_node: Node<T>) -> Tree<T> {
        assert!(new_node.prev_sibling.upgrade().is_none());
        assert!(new_node.next_sibling.is_none());
        assert!(new_node.parent.upgrade().is_none());

        let mut current = self.0.borrow_mut();

        let node_ref = if let Some(prev_sibling_ref) = current.prev_sibling.upgrade() {
            new_node.prev_sibling = Rc::downgrade(&prev_sibling_ref);
            new_node.next_sibling = Some(Rc::clone(&self.0));
            new_node.parent = current.parent.clone();

            let node_ref: NodeRef<T> = new_node.into_ref();
            prev_sibling_ref.borrow_mut().next_sibling = Some(node_ref.clone());
            node_ref
        } else {
            let parent_ref = current.parent.upgrade().expect("Only one element on root allowed.");
            new_node.next_sibling = Some(Rc::clone(&self.0));
            new_node.parent = Rc::downgrade(&parent_ref);

            let node_ref = new_node.into_ref();
            parent_ref.borrow_mut().first_child = Some(Rc::clone(&node_ref));
            node_ref
        };

        current.prev_sibling = Rc::downgrade(&node_ref);

        Tree(node_ref)
    }

    pub fn insert_after(&self, mut new_node: Node<T>) -> Tree<T> {
        assert!(new_node.prev_sibling.upgrade().is_none());
        assert!(new_node.next_sibling.is_none());
        assert!(new_node.parent.upgrade().is_none());

        let mut current = self.0.borrow_mut();

        let node_ref = if let Some(next_sibling_ref) = current.next_sibling.as_ref() {
            new_node.prev_sibling = Rc::downgrade(&self.0);
            new_node.next_sibling = Some(Rc::clone(&next_sibling_ref));
            new_node.parent = current.parent.clone();

            let node_ref = new_node.into_ref();
            next_sibling_ref.borrow_mut().prev_sibling = Rc::downgrade(&node_ref);
            node_ref
        } else {
            let parent_ref = current.parent.upgrade().expect("Only one element on root allowed.");
            new_node.prev_sibling = Rc::downgrade(&self.0);
            new_node.parent = Rc::downgrade(&parent_ref);

            let node_ref = new_node.into_ref();
            parent_ref.borrow_mut().last_child = Rc::downgrade(&node_ref);
            node_ref
        };

        current.next_sibling = Some(Rc::clone(&node_ref));

        Tree(node_ref)
    }

    pub fn remove(&self) {
        self.0.borrow_mut().detach();
    }

    pub fn first_child(&self) -> Option<Tree<T>> {
        self.0
            .borrow()
            .first_child
            .as_ref()
            .map(|node| Tree(Rc::clone(node)))
    }

    pub fn last_child(&self) -> Option<Tree<T>> {
        self.0
            .borrow()
            .last_child
            .upgrade()
            .map(|node| Tree(node))
    }

    pub fn prev_sibling(&self) -> Option<Tree<T>> {
        self.0
            .borrow()
            .prev_sibling
            .upgrade()
            .map(|node| Tree(node))
    }

    pub fn next_sibling(&self) -> Option<Tree<T>> {
        self.0
            .borrow()
            .next_sibling
            .as_ref()
            .map(|node| Tree(Rc::clone(node)))
    }

    pub fn parent(&self) -> Option<Tree<T>> {
        self.0
            .borrow()
            .parent
            .upgrade()
            .map(|node| Tree(node))
    }

    pub fn children(&self) -> impl DoubleEndedIterator<Item = Self> {
        Siblings::new(self.first_child())
    }

    pub fn children_rev(&self) -> impl DoubleEndedIterator<Item = Self> {
        Siblings::new(self.last_child()).rev()
    }

    pub fn next_siblings(&self) -> impl DoubleEndedIterator<Item = Self> {
        Siblings::new(self.next_sibling())
    }

    pub fn prev_siblings(&self) -> impl DoubleEndedIterator<Item = Self> {
        Siblings::new(self.prev_sibling()).rev()
    }

    pub fn descendants<'a>(&'a self) -> impl Iterator<Item = Self> + 'a {
        Descendants::new(self, self.first_child())
    }

    pub fn parents(&self) -> impl Iterator<Item = Self> {
        Parents::new(self.parent())
    }

    pub fn get(&self) -> Ref<T> {
        Ref::map(self.0.borrow(), |node| &node.data)
    }

    pub fn get_mut(&self) -> RefMut<T> {
        RefMut::map(self.0.borrow_mut(), |node| &mut node.data)
    }

    fn ensure_valid_recursive(&self) where T: Debug {
        self.ensure_valid();

        for descendant in self.descendants() {
            descendant.ensure_valid()
        }
    }

    fn ensure_valid(&self) where T: Debug {
        if let Some(prev_sibling) = self.prev_sibling() {
            assert_eq!(prev_sibling.next_sibling(), Some(self.clone()));
        }
        if let Some(last_child) = self.last_child() {
            assert_eq!(last_child.parent(), Some(self.clone()));
        }
        if let Some(first_child) = self.first_child() {
            assert_eq!(first_child.parent(), Some(self.clone()));
        }
        if let Some(next_sibling) = self.next_sibling() {
            assert_eq!(next_sibling.prev_sibling(), Some(self.clone()));
        }
    }
}

impl<T> TryInto<Node<T>> for Tree<T> {
    type Error = Tree<T>;

    fn try_into(self) -> Result<Node<T>, Tree<T>> {
        Rc::try_unwrap(self.0)
            .map_err(Tree)
            .map(|node_cell| node_cell.into_inner())
    }
}

impl<T> Clone for Tree<T> {
    fn clone(&self) -> Self {
        Tree(Rc::clone(&self.0))
    }
}

impl<T> From<Node<T>> for Tree<T> {
    fn from(node: Node<T>) -> Self {
        Tree(node.into_ref())
    }
}

impl<T: Debug> ToString for Tree<T> {
    fn to_string(&self) -> String {
        self.0.borrow().to_string()
    }
}

impl<T> PartialEq for Tree<T> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl<T> Eq for Tree<T> {
}

impl<T> Node<T> {
    pub fn new(data: T) -> Self {
        Self {
            data,
            first_child: None,
            last_child: Weak::new(),
            prev_sibling: Weak::new(),
            next_sibling: None,
            parent: Weak::new(),
        }
    }

    pub fn get(&self) -> &T {
        &self.data
    }

    pub fn get_mut(&mut self) -> &mut T {
        &mut self.data
    }

    fn detach(&mut self) {
        match (self.prev_sibling.upgrade(), self.next_sibling.as_ref()) {
            (Some(prev_sibling_ref), Some(next_sibling_ref)) => {
                let mut prev_sibling = prev_sibling_ref.borrow_mut();
                prev_sibling.next_sibling = Some(next_sibling_ref.clone());

                let mut next_sibling = next_sibling_ref.borrow_mut();
                next_sibling.prev_sibling = Rc::downgrade(&prev_sibling_ref);
            }
            (Some(prev_sibling_ref), None) => {
                prev_sibling_ref.borrow_mut().next_sibling = None;

                if let Some(parent_ref) = self.parent.upgrade() {
                    parent_ref.borrow_mut().last_child = Rc::downgrade(&prev_sibling_ref);
                }
            }
            (None, Some(next_sibling_ref)) => {
                next_sibling_ref.borrow_mut().prev_sibling = Weak::new();

                if let Some(parent_ref) = self.parent.upgrade() {
                    parent_ref.borrow_mut().first_child = Some(next_sibling_ref.clone());
                }
            }
            (None, None) => {
                if let Some(parent_ref) = self.parent.upgrade() {
                    let mut parent = parent_ref.borrow_mut();
                    parent.first_child = None;
                    parent.last_child = Weak::new();
                }
            }
        }

        self.parent = Weak::new();
        self.prev_sibling = Weak::new();
        self.next_sibling = None;
    }

    fn into_ref(self) -> NodeRef<T> {
        Rc::new(RefCell::new(self))
    }
}

impl<T: Debug> ToString for Node<T> {
    fn to_string(&self) -> String {
        fn step<T: Debug>(node: &Node<T>, level: usize) -> String {
            let indent_string = unsafe { String::from_utf8_unchecked(vec![b'\t'; level]) };
            let children_string = match node.first_child {
                Some(ref child_ref) => {
                    format!("\n{}\n{}", step(&child_ref.borrow(), level + 1), indent_string)
                }
                _ => "".to_string()
            };
            let siblings_string = match node.next_sibling {
                Some(ref next_sibling_ref) if level > 0 => {
                    format!("\n{}", step(&next_sibling_ref.borrow(), level))
                }
                _ => "".to_string()
            };
            format!(
                "{}<{:?} data={:?}>{}</{:?}>{}",
                indent_string,
                node as *const Node<T>,
                node.data,
                children_string,
                node as *const Node<T>,
                siblings_string
            )
        }
        step(self, 0)
    }
}

struct Siblings<T> {
    current: Option<Tree<T>>,
}

impl<T> Siblings<T> {
    fn new(current: Option<Tree<T>>) -> Self {
        Self {
            current
        }
    }
}

impl<T> Iterator for Siblings<T> {
    type Item = Tree<T>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.current.take() {
            Some(node) => {
                self.current = node.next_sibling();
                Some(node)
            },
            None => None,
        }
    }
}

impl<T> DoubleEndedIterator for Siblings<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        match self.current.take() {
            Some(current) => {
                self.current = current.prev_sibling();
                Some(current)
            },
            None => None,
        }
    }
}

pub struct Descendants<'a, T> {
    root: &'a Tree<T>,
    current: Option<Tree<T>>,
}

impl<'a, T> Descendants<'a, T> {
    fn new(root: &'a Tree<T>, current: Option<Tree<T>>) -> Self {
        Self {
            root,
            current
        }
    }
}

impl<'a, T> Iterator for Descendants<'a, T> {
    type Item = Tree<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.current.take().map(|current| {
            self.current = current.first_child()
                .or_else(|| current.next_sibling())
                .or_else(|| {
                    let mut current = current.parent();
                    loop {
                        match current {
                            Some(parent) if parent != *self.root => {
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

struct Parents<T> {
    current: Option<Tree<T>>,
}

impl<T> Parents<T> {
    fn new(current: Option<Tree<T>>) -> Self {
        Self {
            current
        }
    }
}

impl<T> Iterator for Parents<T> {
    type Item = Tree<T>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.current.take() {
            Some(current) => {
                self.current = current.parent();
                Some(current)
            },
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_append() {
        let root = Tree::from(Node::new("root"));

        assert_eq!(root.first_child(), None);
        assert_eq!(root.last_child(), None);
        assert_eq!(root.next_sibling(), None);
        assert_eq!(root.prev_sibling(), None);
        assert_eq!(root.parent(), None);
        assert_eq!(*root.get(), "root");

        let foo = root.append(Node::new("foo"));

        assert_eq!(root.first_child(), Some(foo.clone()));
        assert_eq!(root.last_child(), Some(foo.clone()));
        assert_eq!(root.next_sibling(), None);
        assert_eq!(root.prev_sibling(), None);
        assert_eq!(root.parent(), None);
        assert_eq!(*root.get(), "root");

        assert_eq!(foo.first_child(), None);
        assert_eq!(foo.last_child(), None);
        assert_eq!(foo.next_sibling(), None);
        assert_eq!(foo.prev_sibling(), None);
        assert_eq!(foo.parent(), Some(root.clone()));
        assert_eq!(*foo.get(), "foo");

        let bar = root.append(Node::new("bar"));

        assert_eq!(root.first_child(), Some(foo.clone()));
        assert_eq!(root.last_child(), Some(bar.clone()));
        assert_eq!(root.next_sibling(), None);
        assert_eq!(root.prev_sibling(), None);
        assert_eq!(root.parent(), None);
        assert_eq!(*root.get(), "root");

        assert_eq!(foo.first_child(), None);
        assert_eq!(foo.last_child(), None);
        assert_eq!(foo.next_sibling(), Some(bar.clone()));
        assert_eq!(foo.prev_sibling(), None);
        assert_eq!(foo.parent(), Some(root.clone()));
        assert_eq!(*foo.get(), "foo");

        assert_eq!(bar.first_child(), None);
        assert_eq!(bar.last_child(), None);
        assert_eq!(bar.next_sibling(), None);
        assert_eq!(bar.prev_sibling(), Some(foo.clone()));
        assert_eq!(bar.parent(), Some(root.clone()));
        assert_eq!(*bar.get(), "bar");
    }

    #[test]
    fn test_prepend() {
        let root = Tree::from(Node::new("root"));

        assert_eq!(root.first_child(), None);
        assert_eq!(root.last_child(), None);
        assert_eq!(root.prev_sibling(), None);
        assert_eq!(root.next_sibling(), None);
        assert_eq!(root.parent(), None);
        assert_eq!(*root.get(), "root");

        let foo = root.prepend(Node::new("foo"));

        assert_eq!(root.first_child(), Some(foo.clone()));
        assert_eq!(root.last_child(), Some(foo.clone()));
        assert_eq!(root.prev_sibling(), None);
        assert_eq!(root.next_sibling(), None);
        assert_eq!(root.parent(), None);
        assert_eq!(*root.get(), "root");

        assert_eq!(foo.first_child(), None);
        assert_eq!(foo.last_child(), None);
        assert_eq!(foo.prev_sibling(), None);
        assert_eq!(foo.next_sibling(), None);
        assert_eq!(foo.parent(), Some(root.clone()));
        assert_eq!(*foo.get(), "foo");

        let bar = root.prepend(Node::new("bar"));

        assert_eq!(root.first_child(), Some(bar.clone()));
        assert_eq!(root.last_child(), Some(foo.clone()));
        assert_eq!(root.prev_sibling(), None);
        assert_eq!(root.next_sibling(), None);
        assert_eq!(root.parent(), None);
        assert_eq!(*root.get(), "root");

        assert_eq!(foo.first_child(), None);
        assert_eq!(foo.last_child(), None);
        assert_eq!(foo.prev_sibling(), Some(bar.clone()));
        assert_eq!(foo.next_sibling(), None);
        assert_eq!(foo.parent(), Some(root.clone()));
        assert_eq!(*foo.get(), "foo");

        assert_eq!(bar.first_child(), None);
        assert_eq!(bar.last_child(), None);
        assert_eq!(bar.prev_sibling(), None);
        assert_eq!(bar.next_sibling(), Some(foo.clone()));
        assert_eq!(bar.parent(), Some(root.clone()));
        assert_eq!(*bar.get(), "bar");
    }

    #[test]
    fn test_insert_before() {
        let root = Tree::from(Node::new("root"));
        let foo = root.append(Node::new("foo"));
        let bar = root.append(Node::new("bar"));
        let baz = foo.insert_before(Node::new("baz"));
        let qux = foo.insert_before(Node::new("qux"));

        assert_eq!(root.first_child(), Some(baz.clone()));
        assert_eq!(root.last_child(), Some(bar.clone()));
        assert_eq!(root.prev_sibling(), None);
        assert_eq!(root.next_sibling(), None);
        assert_eq!(root.parent(), None);
        assert_eq!(*root.get(), "root");

        assert_eq!(foo.first_child(), None);
        assert_eq!(foo.last_child(), None);
        assert_eq!(foo.prev_sibling(), Some(qux.clone()));
        assert_eq!(foo.next_sibling(), Some(bar.clone()));
        assert_eq!(foo.parent(), Some(root.clone()));
        assert_eq!(*foo.get(), "foo");

        assert_eq!(bar.first_child(), None);
        assert_eq!(bar.last_child(), None);
        assert_eq!(bar.prev_sibling(), Some(foo.clone()));
        assert_eq!(bar.next_sibling(), None);
        assert_eq!(bar.parent(), Some(root.clone()));
        assert_eq!(*bar.get(), "bar");

        assert_eq!(baz.first_child(), None);
        assert_eq!(baz.last_child(), None);
        assert_eq!(baz.prev_sibling(), None);
        assert_eq!(baz.next_sibling(), Some(qux.clone()));
        assert_eq!(baz.parent(), Some(root.clone()));
        assert_eq!(*baz.get(), "baz");

        assert_eq!(qux.first_child(), None);
        assert_eq!(qux.last_child(), None);
        assert_eq!(qux.prev_sibling(), Some(baz.clone()));
        assert_eq!(qux.next_sibling(), Some(foo.clone()));
        assert_eq!(qux.parent(), Some(root.clone()));
        assert_eq!(*qux.get(), "qux");
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

        println!("{}", root.to_string());

        assert_eq!(root.first_child(), Some(foo.clone()));
        assert_eq!(root.last_child(), Some(baz.clone()));
        assert_eq!(root.prev_sibling(), None);
        assert_eq!(root.next_sibling(), None);
        assert_eq!(root.parent(), None);
        assert_eq!(*root.get(), "root");

        assert_eq!(foo.first_child(), None);
        assert_eq!(foo.last_child(), None);
        assert_eq!(foo.prev_sibling(), None);
        assert_eq!(foo.next_sibling(), Some(bar.clone()));
        assert_eq!(foo.parent(), Some(root.clone()));
        assert_eq!(*foo.get(), "foo");

        assert_eq!(bar.first_child(), None);
        assert_eq!(bar.last_child(), None);
        assert_eq!(bar.prev_sibling(), Some(foo.clone()));
        assert_eq!(bar.next_sibling(), Some(qux.clone()));
        assert_eq!(bar.parent(), Some(root.clone()));
        assert_eq!(*bar.get(), "bar");

        assert_eq!(baz.first_child(), None);
        assert_eq!(baz.last_child(), None);
        assert_eq!(baz.prev_sibling(), Some(qux.clone()));
        assert_eq!(baz.next_sibling(), None);
        assert_eq!(baz.parent(), Some(root.clone()));
        assert_eq!(*baz.get(), "baz");

        assert_eq!(qux.first_child(), None);
        assert_eq!(qux.last_child(), None);
        assert_eq!(qux.prev_sibling(), Some(bar.clone()));
        assert_eq!(qux.next_sibling(), Some(baz.clone()));
        assert_eq!(qux.parent(), Some(root.clone()));
        assert_eq!(*qux.get(), "qux");
    }

    #[should_panic]
    #[test]
    fn test_insert_after_should_panic() {
        let root = Tree::from(Node::new("root"));

        root.insert_after(Node::new("foo"));
    }

    #[test]
    fn test_remove() {
        let root = Tree::from(Node::new("root"));
        let foo = root.append(Node::new("foo"));
        let bar = root.append(Node::new("bar"));
        let baz = foo.append(Node::new("baz"));
        let qux = baz.append(Node::new("qux"));
        let quux = qux.append(Node::new("quux"));
        let corge = baz.append(Node::new("corge"));

        bar.remove();

        assert_eq!(root.prev_sibling(), None);
        assert_eq!(root.next_sibling(), None);
        assert_eq!(root.first_child(), Some(foo.clone()));
        assert_eq!(root.last_child(), Some(foo.clone()));
        assert_eq!(root.parent(), None);
        assert_eq!(*root.get(), "root");

        assert_eq!(foo.prev_sibling(), None);
        assert_eq!(foo.next_sibling(), None);
        assert_eq!(foo.first_child(), Some(baz.clone()));
        assert_eq!(foo.last_child(), Some(baz.clone()));
        assert_eq!(foo.parent(), Some(root.clone()));
        assert_eq!(*foo.get(), "foo");

        assert_eq!(bar.prev_sibling(), None);
        assert_eq!(bar.next_sibling(), None);
        assert_eq!(bar.first_child(), None);
        assert_eq!(bar.last_child(), None);
        assert_eq!(bar.parent(), None);
        assert_eq!(*bar.get(), "bar");

        assert_eq!(baz.prev_sibling(), None);
        assert_eq!(baz.next_sibling(), None);
        assert_eq!(baz.first_child(), Some(qux.clone()));
        assert_eq!(baz.last_child(), Some(corge.clone()));
        assert_eq!(baz.parent(), Some(foo.clone()));
        assert_eq!(*baz.get(), "baz");

        assert_eq!(qux.prev_sibling(), None);
        assert_eq!(qux.next_sibling(), Some(corge.clone()));
        assert_eq!(qux.first_child(), Some(quux.clone()));
        assert_eq!(qux.last_child(), Some(quux.clone()));
        assert_eq!(qux.parent(), Some(baz.clone()));
        assert_eq!(*qux.get(), "qux");

        assert_eq!(quux.prev_sibling(), None);
        assert_eq!(quux.next_sibling(), None);
        assert_eq!(quux.first_child(), None);
        assert_eq!(quux.last_child(), None);
        assert_eq!(quux.parent(), Some(qux.clone()));
        assert_eq!(*quux.get(), "quux");

        assert_eq!(corge.prev_sibling(), Some(qux.clone()));
        assert_eq!(corge.next_sibling(), None);
        assert_eq!(corge.first_child(), None);
        assert_eq!(corge.last_child(), None);
        assert_eq!(corge.parent(), Some(baz.clone()));
        assert_eq!(*quux.get(), "quux");
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

        assert_eq!(collect_tree_values(root.children()), vec!["foo", "bar"]);
        assert_eq!(collect_tree_values(foo.children()), vec!["baz"]);
        assert_eq!(collect_tree_values(bar.children()), vec![] as Vec<&str>);
        assert_eq!(collect_tree_values(baz.children()), vec!["qux", "corge"]);
        assert_eq!(collect_tree_values(qux.children()), vec!["quux"]);
        assert_eq!(collect_tree_values(quux.children()), vec![] as Vec<&str>);
        assert_eq!(collect_tree_values(corge.children()), vec![] as Vec<&str>);

        assert_eq!(collect_tree_values(root.children_rev()), vec!["bar", "foo"]);
        assert_eq!(collect_tree_values(foo.children_rev()), vec!["baz"]);
        assert_eq!(collect_tree_values(bar.children_rev()), vec![] as Vec<&str>);
        assert_eq!(collect_tree_values(baz.children_rev()), vec!["corge", "qux"]);
        assert_eq!(collect_tree_values(qux.children_rev()), vec!["quux"]);
        assert_eq!(collect_tree_values(quux.children_rev()), vec![] as Vec<&str>);
        assert_eq!(collect_tree_values(corge.children_rev()), vec![] as Vec<&str>);
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

        assert_eq!(collect_tree_values(root.descendants()), vec!["foo", "baz", "qux", "quux", "corge", "bar"]);
        assert_eq!(collect_tree_values(foo.descendants()), vec!["baz", "qux", "quux", "corge"]);
        assert_eq!(collect_tree_values(bar.descendants()), vec![] as Vec<&str>);
        assert_eq!(collect_tree_values(baz.descendants()), vec!["qux", "quux", "corge"]);
        assert_eq!(collect_tree_values(qux.descendants()), vec!["quux"]);
        assert_eq!(collect_tree_values(quux.descendants()), vec![] as Vec<&str>);
        assert_eq!(collect_tree_values(corge.descendants()), vec![] as Vec<&str>);
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

        assert_eq!(collect_tree_values(root.parents()), vec![] as Vec<&str>);
        assert_eq!(collect_tree_values(foo.parents()), vec!["root"]);
        assert_eq!(collect_tree_values(bar.parents()), vec!["root"]);
        assert_eq!(collect_tree_values(baz.parents()), vec!["foo", "root"]);
        assert_eq!(collect_tree_values(qux.parents()), vec!["baz", "foo", "root"]);
        assert_eq!(collect_tree_values(quux.parents()), vec!["qux", "baz", "foo", "root"]);
        assert_eq!(collect_tree_values(corge.parents()), vec!["baz", "foo", "root"]);
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

        assert_eq!(collect_tree_values(root.prev_siblings()), vec![] as Vec<&str>);
        assert_eq!(collect_tree_values(foo.prev_siblings()), vec![] as Vec<&str>);
        assert_eq!(collect_tree_values(bar.prev_siblings()), vec!["foo"]);
        assert_eq!(collect_tree_values(baz.prev_siblings()), vec!["bar", "foo"]);
        assert_eq!(collect_tree_values(qux.prev_siblings()), vec!["baz", "bar", "foo"]);
        assert_eq!(collect_tree_values(quux.prev_siblings()), vec!["qux", "baz", "bar", "foo"]);
        assert_eq!(collect_tree_values(corge.prev_siblings()), vec!["quux", "qux", "baz", "bar", "foo"]);

        assert_eq!(collect_tree_values(root.next_siblings()), vec![] as Vec<&str>);
        assert_eq!(collect_tree_values(foo.next_siblings()), vec!["bar", "baz", "qux", "quux", "corge"]);
        assert_eq!(collect_tree_values(bar.next_siblings()), vec!["baz", "qux", "quux", "corge"]);
        assert_eq!(collect_tree_values(baz.next_siblings()), vec!["qux", "quux", "corge"]);
        assert_eq!(collect_tree_values(qux.next_siblings()), vec!["quux", "corge"]);
        assert_eq!(collect_tree_values(quux.next_siblings()), vec!["corge"]);
        assert_eq!(collect_tree_values(corge.next_siblings()), vec![] as Vec<&str>);
    }

    fn collect_tree_values<T: Clone>(iter: impl Iterator<Item = Tree<T>>) -> Vec<T> {
        iter.map(|tree| tree.get().clone()).collect::<Vec<_>>()
    }
}
