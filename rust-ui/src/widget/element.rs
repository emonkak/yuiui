use std::array;
use std::fmt;
use std::sync::Arc;

use super::{PolymophicWidget, Widget, WidgetMeta};

#[derive(Debug)]
pub struct Element<Renderer> {
    pub widget: BoxedWidget<Renderer>,
    pub children: Children<Renderer>,
    pub key: Option<Key>,
}

#[derive(Debug)]
pub enum Child<Renderer> {
    Multiple(Vec<Element<Renderer>>),
    Single(Element<Renderer>),
    None,
}

pub type BoxedWidget<Renderer> = Arc<dyn PolymophicWidget<Renderer>>;

pub type Children<Renderer> = Arc<Vec<Element<Renderer>>>;

pub type Key = usize;

pub trait IntoElement<Renderer> {
    fn into_element(self, children: Children<Renderer>) -> Element<Renderer>;
}

impl<Renderer> Element<Renderer> {
    pub fn build<const N: usize>(
        widget: impl IntoElement<Renderer> + 'static,
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

        widget.into_element(Arc::new(flatten_children))
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
