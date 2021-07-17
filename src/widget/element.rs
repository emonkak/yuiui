use std::array;
use std::fmt;

use super::{BoxedWidget, Widget, WidgetMeta};

pub struct Element<Handle> {
    pub widget: BoxedWidget<Handle>,
    pub children: Children<Handle>,
    pub key: Option<Key>,
}

pub trait IntoElement<Handle> {
    fn into_element(self, children: Children<Handle>) -> Element<Handle>;
}

pub type Key = usize;

pub type Children<Handle> = Box<[Element<Handle>]>;

#[derive(Debug)]
pub enum Child<Handle> {
    Multiple(Vec<Element<Handle>>),
    Single(Element<Handle>),
    None,
}

impl<Handle> Element<Handle> {
    pub fn new<State: 'static>(
        widget: impl Widget<Handle, State = State> + 'static,
        children: Children<Handle>,
        key: Option<Key>,
    ) -> Self {
        Self {
            widget: Box::new(widget),
            children,
            key,
        }
    }

    pub fn build<const N: usize>(
        widget: impl IntoElement<Handle> + 'static,
        children: [Child<Handle>; N],
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

        widget.into_element(flatten_children.into_boxed_slice())
    }

    fn fmt_rec(&self, f: &mut fmt::Formatter<'_>, level: usize) -> fmt::Result {
        let name = self.widget.name();
        let indent_str = unsafe { String::from_utf8_unchecked(vec![b'\t'; level]) };
        if self.children.len() > 0 {
            write!(f, "{}<{}>", indent_str, name)?;
            for i in 0..self.children.len() {
                write!(f, "\n")?;
                self.children[i].fmt_rec(f, level + 1)?
            }
            write!(f, "\n{}</{}>", indent_str, name)?;
        } else {
            write!(f, "{}<{}></{}>", indent_str, name, name)?;
        }
        Ok(())
    }
}

impl<Handle> fmt::Display for Element<Handle> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_rec(f, 0)
    }
}

impl<Handle> fmt::Debug for Element<Handle> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Element")
            .field("widget", &self.widget)
            .field("children", &self.children)
            .finish()
    }
}

impl<Handle> Into<Vec<Element<Handle>>> for Child<Handle> {
    fn into(self) -> Vec<Element<Handle>> {
        match self {
            Child::None => Vec::new(),
            Child::Single(element) => vec![element],
            Child::Multiple(elements) => elements,
        }
    }
}

impl<Handle> From<Vec<Element<Handle>>> for Child<Handle> {
    fn from(elements: Vec<Element<Handle>>) -> Self {
        Child::Multiple(elements)
    }
}

impl<Handle> From<Option<Element<Handle>>> for Child<Handle> {
    fn from(element: Option<Element<Handle>>) -> Self {
        match element {
            Some(element) => Child::Single(element),
            None => Child::None,
        }
    }
}

impl<Handle> From<Element<Handle>> for Child<Handle> {
    fn from(element: Element<Handle>) -> Self {
        Child::Single(element)
    }
}

impl<Handle, State: 'static, W: Widget<Handle, State = State> + WidgetMeta + 'static> From<W>
    for Child<Handle>
{
    fn from(widget: W) -> Self {
        Child::Single(Element {
            widget: Box::new(widget),
            children: Box::new([]),
            key: None,
        })
    }
}

#[macro_export]
macro_rules! element {
    ($expr:expr => { $($content:tt)* }) => {
        $crate::widget::element::Element::build($expr, __element_children!([] $($content)*))
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
    ([$($children:expr)*] $expr:expr; $($rest:tt)*) => {
        __element_children!([$($children)* $crate::widget::element::Child::from($expr)] $($rest)*)
    };
    ([$($children:expr)*] $expr:expr) => {
        __element_children!([$($children)* $crate::widget::element::Child::from($expr)])
    };
    ([$($children:expr)*] $expr:expr) => {
        __element_children!([$($children)* $crate::widget::element::Child::from($expr)])
    };
    ([$($children:expr)*]) => {
        [$($children),*]
    };
}
