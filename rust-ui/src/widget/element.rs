use std::array;
use std::fmt;
use std::sync::Arc;

use crate::support::tree::{Link, NodeId, Tree};

use super::null::Null;
use super::widget::{PolymophicWidget, Widget, WidgetMeta};

#[derive(Debug)]
pub struct Element<Renderer> {
    pub widget: Arc<dyn PolymophicWidget<Renderer>>,
    pub children: Children<Renderer>,
    pub key: Option<Key>,
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

pub type Children<Renderer> = Arc<Vec<Element<Renderer>>>;

pub type Key = usize;

pub trait BuildElement<Renderer> {
    fn build_element(self, children: Children<Renderer>) -> Element<Renderer>;
}

impl<Renderer> Element<Renderer> {
    pub fn new<Widget>(
        widget: Widget,
        children: impl Into<Children<Renderer>>,
        key: Option<Key>,
    ) -> Self
    where
        Widget: self::Widget<Renderer> + Send + 'static,
        Widget::State: 'static,
        Widget::Message: 'static,
        Widget::PaintObject: 'static,
    {
        Self {
            widget: Arc::new(widget),
            children: children.into(),
            key,
        }
    }

    pub fn build<const N: usize>(
        widget: impl BuildElement<Renderer> + 'static,
        children: [Child<Renderer>; N],
    ) -> Self {
        let mut flatten_children = Vec::with_capacity(N);

        for child in array::IntoIter::new(children) {
            match child {
                Child::Multiple(elements) => {
                    for element in elements {
                        flatten_children.push(element)
                    }
                }
                Child::Single(element) => flatten_children.push(element),
                _ => {}
            }
        }

        widget.build_element(Arc::new(flatten_children))
    }
}

impl<Renderer> Clone for Element<Renderer> {
    fn clone(&self) -> Self {
        Self {
            widget: Arc::clone(&self.widget),
            children: Arc::clone(&self.children),
            key: self.key,
        }
    }
}

impl<Renderer> fmt::Display for Element<Renderer> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn fmt_rec<Renderer>(
            this: &Element<Renderer>,
            f: &mut fmt::Formatter<'_>,
            level: usize,
        ) -> fmt::Result {
            let indent_str = unsafe { String::from_utf8_unchecked(vec![b'\t'; level]) };
            if this.children.len() > 0 {
                write!(f, "{}<{:?}>", indent_str, this.widget)?;
                for i in 0..this.children.len() {
                    write!(f, "\n")?;
                    fmt_rec(&this.children[i], f, level + 1)?;
                }
                write!(f, "\n{}</{:?}>", indent_str, this.widget)?;
            } else {
                write!(f, "{}<{:?}></{:?}>", indent_str, this.widget, this.widget)?;
            }
            Ok(())
        }

        fmt_rec(self, f, 0)
    }
}

impl<Renderer> From<Element<Renderer>> for Children<Renderer> {
    fn from(element: Element<Renderer>) -> Self {
        Arc::new(vec![element])
    }
}

impl<Renderer> From<Vec<Element<Renderer>>> for Child<Renderer> {
    fn from(elements: Vec<Element<Renderer>>) -> Self {
        Child::Multiple(elements)
    }
}

impl<Renderer> From<Option<Element<Renderer>>> for Child<Renderer> {
    fn from(element: Option<Element<Renderer>>) -> Self {
        match element {
            Some(element) => Child::Single(element),
            None => Child::None,
        }
    }
}

impl<Renderer> From<Element<Renderer>> for Child<Renderer> {
    fn from(element: Element<Renderer>) -> Self {
        Child::Single(element)
    }
}

impl<Renderer, Widget> From<Widget> for Child<Renderer>
where
    Widget: self::Widget<Renderer> + WidgetMeta + 'static,
    Widget::State: 'static,
    Widget::Message: 'static,
    Widget::PaintObject: 'static,
{
    fn from(widget: Widget) -> Self {
        Child::Single(Element {
            widget: Arc::new(widget),
            children: Arc::new(Vec::new()),
            key: None,
        })
    }
}

pub fn create_element_tree<Renderer>() -> (ElementTree<Renderer>, ElementId) {
    let mut tree = Tree::new();
    let root_id = tree.attach(Element::new(Null, Vec::new(), None));
    (tree, root_id)
}

#[macro_export]
macro_rules! element {
    ($expr:expr => { $($content:tt)* }) => {
        $crate::widget::element::Element::build($expr, __element_children!([] $($content)*))
    };
    ($expr:expr => $child:expr) => {
        element!($expr => { $child })
    };
    ($expr:expr) => {
        $crate::widget::element::Element::build($expr, [])
    };
}

#[macro_export]
macro_rules! __element_children {
    ([$($children:expr)*] $expr:expr => { $($content:tt)* } $($rest:tt)*) => {
        __element_children!([$($children)* $crate::widget::element::Child::Single($crate::widget::element::Element::build($expr, __element_children!([] $($content)*)))] $($rest)*)
    };
    ([$($children:expr)*] $expr:expr => $child:expr, $($rest:tt)*) => {
        __element_children!([$($children)*] $expr => { $child } $($rest)*)
    };
    ([$($children:expr)*] $expr:expr => $child:expr) => {
        __element_children!([$($children)*] $expr => { $child })
    };
    ([$($children:expr)*] $expr:expr, $($rest:tt)*) => {
        __element_children!([$($children)* $crate::widget::element::Child::from($expr)] $($rest)*)
    };
    ([$($children:expr)*] $expr:expr) => {
        __element_children!([$($children)* $crate::widget::element::Child::from($expr)])
    };
    ([$($children:expr)*]) => {
        [$($children),*]
    };
}
