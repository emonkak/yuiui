use std::sync::Arc;

use crate::support::tree::{Link, NodeId, Tree};

use super::null::Null;
use super::widget::{AsAny, PolymophicWidget, Widget};

#[derive(Debug)]
pub struct Element<Renderer> {
    pub widget: Arc<dyn PolymophicWidget<Renderer>>,
    pub key: Option<Key>,
}

pub struct WithKey<Inner> {
    pub inner: Inner,
    pub key: Key,
}

#[derive(Debug)]
pub enum Child<Renderer> {
    Multiple(Vec<Element<Renderer>>),
    Single(Element<Renderer>),
    None,
}

#[derive(Debug)]
pub enum Patch<Renderer> {
    Append(ElementId, Element<Renderer>),
    Insert(ElementId, Element<Renderer>),
    Update(ElementId, Element<Renderer>),
    Move(ElementId, ElementId),
    Remove(ElementId),
}

pub type ElementId = NodeId;

pub type ElementTree<Renderer> = Tree<Element<Renderer>>;

pub type ElementNode<Renderer> = Link<Element<Renderer>>;

pub type Children<Renderer> = Vec<Element<Renderer>>;

pub type Key = usize;

pub trait IntoElement<R> {
    fn into_element(self) -> Element<R>;
}

impl<R> Element<R> {
    pub fn new<Widget>(
        widget: Widget,
        key: Option<Key>,
    ) -> Self
    where
        Widget: self::Widget<R> + Send + 'static,
        Widget::State: 'static,
        Widget::Message: 'static,
    {
        Self {
            widget: Arc::new(widget),
            key,
        }
    }
}

impl<R> Clone for Element<R> {
    fn clone(&self) -> Self {
        Self {
            widget: Arc::clone(&self.widget),
            key: self.key,
        }
    }
}

impl<W, R> IntoElement<R> for W
where
    W: Widget<R> + AsAny + 'static,
    W::State: 'static,
    W::Message: 'static,
{
    fn into_element(self) -> Element<R> {
        Element {
            widget: Arc::new(self),
            key: None
        }
    }
}

impl<W, R> IntoElement<R> for WithKey<W>
where
    W: Widget<R> + AsAny + 'static,
    W::State: 'static,
    W::Message: 'static,
{
    fn into_element(self) -> Element<R> {
        Element {
            widget: Arc::new(self.inner),
            key: Some(self.key)
        }
    }
}

impl<R> From<Element<R>> for Children<R> {
    fn from(element: Element<R>) -> Self {
        vec![element]
    }
}

// impl<Renderer> fmt::Display for Element<Renderer> {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         fn fmt_rec<Renderer>(
//             this: &Element<Renderer>,
//             f: &mut fmt::Formatter<'_>,
//             level: usize,
//         ) -> fmt::Result {
//             let indent_str = unsafe { String::from_utf8_unchecked(vec![b'\t'; level]) };
//             if this.children.len() > 0 {
//                 write!(f, "{}<{:?}>", indent_str, this.widget)?;
//                 for i in 0..this.children.len() {
//                     write!(f, "\n")?;
//                     fmt_rec(&this.children[i], f, level + 1)?;
//                 }
//                 write!(f, "\n{}</{:?}>", indent_str, this.widget)?;
//             } else {
//                 write!(f, "{}<{:?}></{:?}>", indent_str, this.widget, this.widget)?;
//             }
//             Ok(())
//         }
//
//         fmt_rec(self, f, 0)
//     }
// }

pub fn create_element_tree<R: 'static>() -> (ElementTree<R>, ElementId) {
    let mut tree = Tree::new();
    let root_id = tree.attach(Element::new(Null { children: Vec::new() }, None));
    (tree, root_id)
}
