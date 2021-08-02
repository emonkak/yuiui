use std::array;
use std::fmt;
use std::sync::Arc;

use super::{PolymophicWidget, Widget, WidgetMeta};

#[derive(Debug)]
pub struct Element<Painter> {
    pub widget: BoxedWidget<Painter>,
    pub children: Children<Painter>,
    pub key: Option<Key>,
}

#[derive(Debug)]
pub enum Child<Painter> {
    Multiple(Vec<Element<Painter>>),
    Single(Element<Painter>),
    None,
}

pub type BoxedWidget<Painter> = Arc<dyn PolymophicWidget<Painter>>;

pub type Children<Painter> = Arc<Vec<Element<Painter>>>;

pub type Key = usize;

pub trait IntoElement<Painter> {
    fn into_element(self, children: Children<Painter>) -> Element<Painter>;
}

impl<Painter> Element<Painter> {
    pub fn build<const N: usize>(
        widget: impl IntoElement<Painter> + 'static,
        children: [Child<Painter>; N],
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

        widget.into_element(Arc::new(flatten_children))
    }
}

impl<Painter> Clone for Element<Painter> {
    fn clone(&self) -> Self {
        Self {
            widget: Arc::clone(&self.widget),
            children: Arc::clone(&self.children),
            key: self.key,
        }
    }
}

impl<Painter> fmt::Display for Element<Painter> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn fmt_rec<Painter>(
            this: &Element<Painter>,
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

impl<Painter> From<Element<Painter>> for Children<Painter> {
    fn from(element: Element<Painter>) -> Self {
        Arc::new(vec![element])
    }
}

impl<Painter> From<Vec<Element<Painter>>> for Child<Painter> {
    fn from(elements: Vec<Element<Painter>>) -> Self {
        Child::Multiple(elements)
    }
}

impl<Painter> From<Option<Element<Painter>>> for Child<Painter> {
    fn from(element: Option<Element<Painter>>) -> Self {
        match element {
            Some(element) => Child::Single(element),
            None => Child::None,
        }
    }
}

impl<Painter> From<Element<Painter>> for Child<Painter> {
    fn from(element: Element<Painter>) -> Self {
        Child::Single(element)
    }
}

impl<Painter, Widget> From<Widget> for Child<Painter>
where
    Widget: self::Widget<Painter> + WidgetMeta + 'static,
    Widget::State: 'static,
{
    fn from(widget: Widget) -> Self {
        Child::Single(Element {
            widget: Arc::new(widget),
            children: Arc::new(Vec::new()),
            key: None,
        })
    }
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
