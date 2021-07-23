use std::array;
use std::fmt;

use super::{BoxedWidget, Widget, WidgetMeta};

#[derive(Debug)]
pub struct Element<Handle> {
    pub widget: BoxedWidget<Handle>,
    pub children: Children<Handle>,
    pub key: Option<Key>,
}

#[derive(Debug)]
pub enum Child<Handle> {
    Multiple(Vec<Element<Handle>>),
    Single(Element<Handle>),
    None,
}

pub type Key = usize;

pub type Children<Handle> = Vec<Element<Handle>>;

pub trait IntoElement<Handle> {
    fn into_element(self, children: Children<Handle>) -> Element<Handle>;
}

impl<Handle> Element<Handle> {
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

        widget.into_element(flatten_children)
    }
}

impl<Handle> fmt::Display for Element<Handle> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn fmt_rec<Handle>(
            this: &Element<Handle>,
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

impl<Handle> Into<Vec<Element<Handle>>> for Child<Handle> {
    fn into(self) -> Vec<Element<Handle>> {
        match self {
            Child::Single(element) => vec![element],
            Child::Multiple(elements) => elements,
            Child::None => Vec::new(),
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

impl<Handle, Widget> From<Widget> for Child<Handle>
where
    Widget: self::Widget<Handle> + WidgetMeta + 'static,
    Widget::State: 'static,
{
    fn from(widget: Widget) -> Self {
        Child::Single(Element {
            widget: Box::new(widget),
            children: Vec::new(),
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
    ([$($children:expr)*]) => {
        [$($children),*]
    };
}
