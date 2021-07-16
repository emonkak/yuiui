pub mod fill;
pub mod flex;
pub mod null;
pub mod padding;

use std::any::{self, Any};
use std::array;
use std::fmt;

use geometrics::{Rectangle, Size};
use layout::{BoxConstraints, LayoutContext, LayoutResult};
use lifecycle::{Lifecycle, LifecycleContext};
use paint::PaintContext;
use tree::{NodeId, Link, Tree};

pub type WidgetTree<Handle> = Tree<BoxedWidget<Handle>>;

pub type WidgetNode<Handle> = Link<BoxedWidget<Handle>>;

pub type BoxedWidget<Handle> = Box<dyn DynamicWidget<Handle>>;

pub type Key = usize;

pub struct Element<Handle> {
    pub widget: BoxedWidget<Handle>,
    pub children: Box<[Element<Handle>]>,
}

#[derive(Debug)]
pub enum Child<Handle> {
    Multiple(Vec<Element<Handle>>),
    Single(Element<Handle>),
    None,
}

pub trait Widget<Handle>: WidgetMeta {
    type State;

    fn initial_state(&self) -> Self::State;

    #[inline]
    fn should_update(&self, _next_widget: &Self, _state: &Self::State) -> bool {
        true
    }

    #[inline]
    fn lifecycle(&self, _lifecycle: Lifecycle<&Self>, _state: &mut Self::State, _context: &mut LifecycleContext) {
    }

    #[inline]
    fn render(&self, children: Box<[Element<Handle>]>, _state: &mut Self::State) -> Box<[Element<Handle>]> {
        children
    }

    #[inline]
    fn layout(
        &self,
        node_id: NodeId,
        box_constraints: BoxConstraints,
        response: Option<(NodeId, Size)>,
        tree: &WidgetTree<Handle>,
        _state: &mut Self::State,
        _context: &mut dyn LayoutContext
    ) -> LayoutResult {
        if let Some((_, size)) = response {
            LayoutResult::Size(size)
        } else {
            if let Some(child_id) = tree[node_id].first_child() {
                LayoutResult::RequestChild(child_id, box_constraints)
            } else {
                LayoutResult::Size(box_constraints.max)
            }
        }
    }

    #[inline]
    fn paint(&self, _handle: &Handle, _rectangle: &Rectangle, _state: &mut Self::State, _paint_context: &mut dyn PaintContext<Handle>) {
    }
}

pub trait DynamicWidget<Handle>: WidgetMeta {
    fn initial_state(&self) -> Box<dyn Any>;

    fn should_update(&self, new_widget: &dyn DynamicWidget<Handle>, state: &dyn Any) -> bool;

    fn lifecycle(&self, lifecycle: Lifecycle<&dyn DynamicWidget<Handle>>, state: &mut dyn Any, context: &mut LifecycleContext);

    fn render(&self, children: Box<[Element<Handle>]>, state: &mut dyn Any) -> Box<[Element<Handle>]>;

    fn layout(
        &self,
        node_id: NodeId,
        box_constraints: BoxConstraints,
        response: Option<(NodeId, Size)>,
        tree: &WidgetTree<Handle>,
        _state: &mut dyn Any,
        _context: &mut dyn LayoutContext
    ) -> LayoutResult;

    fn paint(&self, handle: &Handle, rectangle: &Rectangle, state: &mut dyn Any, paint_context: &mut dyn PaintContext<Handle>);
}

pub trait WidgetMeta {
    #[inline(always)]
    fn name(&self) -> &'static str {
        any::type_name::<Self>()
    }

    #[inline(always)]
    fn key(&self) -> Option<Key> {
        None
    }

    #[inline(always)]
    fn with_key(self, key: Key) -> WithKey<Self> where Self: Sized {
        WithKey {
            inner: self,
            key
        }
    }

    fn as_any(&self) -> &dyn Any;
}

pub struct WithKey<T> {
    inner: T,
    key: Key,
}

impl<Handle> Element<Handle> {
    pub fn new<State: 'static, const N: usize>(widget: impl Widget<Handle, State=State> + 'static, children: [Element<Handle>; N]) -> Self {
        Self {
            widget: Box::new(widget),
            children: Box::new(children),
        }
    }

    pub fn build<State: 'static, const N: usize>(widget: impl Widget<Handle, State=State> + 'static, children: [Child<Handle>; N]) -> Self {
        let mut flatten_children = Vec::with_capacity(N);

        for child in array::IntoIter::new(children) {
            match child {
                Child::Multiple(elements) => {
                    for element in elements {
                        flatten_children.push(element)
                    }
                }
                Child::Single(element) => {
                    flatten_children.push(element)
                }
                _ => {}
            }
        }

        Self {
            widget: Box::new(widget),
            children: flatten_children.into_boxed_slice(),
        }
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

impl<Handle, State: 'static, W: Widget<Handle, State=State> + WidgetMeta + 'static> From<W> for Child<Handle> {
    fn from(widget: W) -> Self {
        Child::Single(Element {
            widget: Box::new(widget),
            children: Box::new([])
        })
    }
}

impl<Handle> fmt::Debug for dyn DynamicWidget<Handle> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl<Handle> fmt::Display for dyn DynamicWidget<Handle> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl<Handle, State: 'static, T: Widget<Handle, State=State> + WidgetMeta + 'static> DynamicWidget<Handle> for T {
    #[inline]
    fn initial_state(&self) -> Box<dyn Any> {
        Box::new(self.initial_state())
    }

    #[inline]
    fn should_update(&self, new_widget: &dyn DynamicWidget<Handle>, state: &dyn Any) -> bool {
        self.should_update(
            new_widget.as_any().downcast_ref::<Self>().unwrap(),
            state.downcast_ref().unwrap()
        )
    }

    #[inline]
    fn lifecycle(&self, lifecycle: Lifecycle<&dyn DynamicWidget<Handle>>, state: &mut dyn Any, context: &mut LifecycleContext) {
        self.lifecycle(
            lifecycle.map(|widget| widget.as_any().downcast_ref().unwrap()),
            state.downcast_mut().unwrap(),
            context
        );
    }

    #[inline]
    fn render(&self, children: Box<[Element<Handle>]>, state: &mut dyn Any) -> Box<[Element<Handle>]> {
        self.render(children, state.downcast_mut().unwrap())
    }

    #[inline]
    fn layout(
        &self,
        node_id: NodeId,
        box_constraints: BoxConstraints,
        response: Option<(NodeId, Size)>,
        tree: &WidgetTree<Handle>,
        state: &mut dyn Any,
        context: &mut dyn LayoutContext
    ) -> LayoutResult {
        self.layout(
            node_id,
            box_constraints,
            response,
            tree,
            state.downcast_mut().unwrap(),
            context
        )
    }

    #[inline]
    fn paint(&self, handle: &Handle, rectangle: &Rectangle, state: &mut dyn Any, paint_context: &mut dyn PaintContext<Handle>) {
        self.paint(handle, rectangle, state.downcast_mut().unwrap(), paint_context)
    }
}

impl<Handle, T: Widget<Handle> + 'static> Widget<Handle> for WithKey<T> {
    type State = T::State;

    #[inline]
    fn initial_state(&self) -> Self::State {
        self.inner.initial_state()
    }

    #[inline]
    fn should_update(&self, new_widget: &Self, state: &Self::State) -> bool {
        self.inner.should_update(&new_widget.inner, state)
    }

    #[inline]
    fn render(&self, children: Box<[Element<Handle>]>, state: &mut Self::State) -> Box<[Element<Handle>]> {
        self.inner.render(children, state)
    }

    #[inline]
    fn layout(
        &self,
        node_id: NodeId,
        box_constraints: BoxConstraints,
        response: Option<(NodeId, Size)>,
        tree: &WidgetTree<Handle>,
        state: &mut Self::State,
        context: &mut dyn LayoutContext
    ) -> LayoutResult {
        self.inner.layout(node_id, box_constraints, response, tree, state, context)
    }

    #[inline(always)]
    fn paint(&self, handle: &Handle, rectangle: &Rectangle, state: &mut Self::State, paint_context: &mut dyn PaintContext<Handle>) {
        self.inner.paint(handle, rectangle, state, paint_context)
    }
}

impl<T: WidgetMeta> WidgetMeta for WithKey<T> {
    #[inline(always)]
    fn name(&self) -> &'static str {
        self.inner.name()
    }

    #[inline(always)]
    fn key(&self) -> Option<Key> {
        Some(self.key)
    }

    #[inline(always)]
    fn as_any(&self) -> &dyn Any {
        self.inner.as_any()
    }
}

#[macro_export]
macro_rules! element {
    ($expr:expr => { $($content:tt)* }) => {
        $crate::widget::Element::build($expr, __element_children!([] $($content)*))
    };
    ($expr:expr) => {
        $crate::widget::Element::build($expr, [])
    };
}

#[macro_export]
macro_rules! __element_children {
    ([$($children:expr)*] $expr:expr => { $($content:tt)* } $($rest:tt)*) => {
        __element_children!([$($children)* $crate::widget::Child::Single($crate::widget::Element::build($expr, __element_children!([] $($content)*)))] $($rest)*)
    };
    ([$($children:expr)*] $expr:expr; $($rest:tt)*) => {
        __element_children!([$($children)* $crate::widget::Child::from($expr)] $($rest)*)
    };
    ([$($children:expr)*] $expr:expr) => {
        __element_children!([$($children)* $crate::widget::Child::from($expr)])
    };
    ([$($children:expr)*]) => {
        [$($children),*]
    };
}
