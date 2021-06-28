use std::any;
use std::array;
use std::fmt;

use geometrics::{Rectangle, Size};
use layout::{BoxConstraints, LayoutResult, LayoutContext};
use paint::PaintContext;
use tree::{Node, NodeId, Tree};

pub type FiberTree<Window> = Tree<Fiber<Window>>;

pub type FiberNode<Window> = Node<Fiber<Window>>;

#[derive(Debug)]
pub struct Fiber<Window> {
    pub(crate) widget: Box<dyn Widget<Window>>,
    pub(crate) rendered_children: Option<Box<[Element<Window>]>>,
    pub(crate) handle: Option<Window>,
    pub(crate) state: Option<Box<dyn any::Any>>,
    pub(crate) dirty: bool,
}

pub struct Element<Window> {
    pub widget: Box<dyn Widget<Window>>,
    pub children: Box<[Element<Window>]>,
}

#[derive(Debug)]
pub enum Child<Window> {
    Multiple(Vec<Element<Window>>),
    Single(Element<Window>),
    Empty,
}

pub trait Widget<Window>: WidgetMeta {
    fn should_update(&self, _next_widget: &dyn Widget<Window>, _next_children: &[Element<Window>]) -> bool {
        true
    }

    fn render(&self, children: Box<[Element<Window>]>) -> Box<[Element<Window>]> {
        children
    }

    fn mount(&mut self, _parent_handle: &Window, _rectangle: &Rectangle) -> Option<Window> {
        None
    }

    fn unmount(&mut self, _handle: &Window) {
    }

    fn layout(
        &mut self,
        node_id: NodeId,
        box_constraints: BoxConstraints,
        response: Option<(NodeId, Size)>,
        fiber_tree: &FiberTree<Window>,
        _layout_context: &mut LayoutContext,
    ) -> LayoutResult {
        if let Some((_, size)) = response {
            LayoutResult::Size(size)
        } else {
            if let Some(child_id) = fiber_tree[node_id].first_child() {
                LayoutResult::RequestChild(child_id, box_constraints)
            } else {
                LayoutResult::Size(box_constraints.max)
            }
        }
    }

    fn paint(&mut self, _handle: &Window, _rectangle: &Rectangle, _paint_context: &mut PaintContext<Window>) {
    }
}

pub trait WidgetMeta {
    #[inline]
    fn name(&self) -> &'static str {
        let full_name = any::type_name::<Self>();
        full_name
            .rsplit_once("::")
            .map(|(_, last)| last)
            .unwrap_or(full_name)
    }

    #[inline]
    fn key(&self) -> Option<Key> {
        None
    }

    #[inline]
    fn with_key(self, key: Key) -> WithKey<Self> where Self: Sized {
        WithKey {
            inner: self,
            key
        }
    }

    fn as_any(&self) -> &dyn any::Any;
}

pub struct WithKey<Inner> {
    inner: Inner,
    key: Key,
}

pub type Key = usize;

impl<Window> Fiber<Window> {
    pub(crate) fn new(element: Element<Window>) -> Fiber<Window> {
        let rendered_children = element.widget.render(element.children);
        Fiber {
            widget: element.widget,
            rendered_children: Some(rendered_children),
            handle: None,
            state: None,
            dirty: true,
        }
    }

    pub fn coerce_widget<T: Widget<Window> + 'static>(&self) -> Option<&T> {
        self.widget.as_any().downcast_ref()
    }

    pub fn get_state<T: 'static>(&self) -> Option<&T> {
        self.state.as_ref().and_then(|state| state.downcast_ref())
    }

    pub fn set_state<T: 'static>(&mut self, new_state: T) {
        self.state = Some(Box::new(new_state));
        self.dirty = true;
    }

    pub fn update_state<T: 'static>(&mut self, updater: impl FnOnce(Option<&T>) -> T) -> &T {
        self.state = Some(Box::new(updater(self.get_state())));
        self.dirty = true;
        self.get_state().unwrap()
    }

    pub(crate) fn update(&mut self, element: Element<Window>) -> bool {
        if self.widget.should_update(&*element.widget, &*element.children) {
            let rendered_children = element.widget.render(element.children);
            self.widget = element.widget;
            self.dirty = true;
            self.rendered_children = Some(rendered_children);
            true
        } else {
            false
        }
    }

    pub(crate) fn unmount(&mut self) {
        let widget = &mut self.widget;
        if let Some(handle) = self.handle.as_ref() {
            widget.unmount(handle);
        }
    }
}

impl<Window> fmt::Display for Fiber<Window> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.widget.name())
    }
}

impl<Window> Element<Window> {
    pub fn new<const N: usize>(widget: impl Widget<Window> + 'static, children: [Element<Window>; N]) -> Self {
        Self {
            widget: Box::new(widget),
            children: Box::new(children),
        }
    }

    pub fn build<const N: usize>(widget: impl Widget<Window> + 'static, children: [Child<Window>; N]) -> Self {
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

impl<Window> fmt::Display for Element<Window> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_rec(f, 0)
    }
}

impl<Window> fmt::Debug for Element<Window> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Element")
            .field("widget", &self.widget)
            .field("children", &self.children)
            .finish()
    }
}

impl<Window> From<Vec<Element<Window>>> for Child<Window> {
    fn from(elements: Vec<Element<Window>>) -> Self {
        Child::Multiple(elements)
    }
}

impl<Window> From<Option<Element<Window>>> for Child<Window> {
    fn from(element: Option<Element<Window>>) -> Self {
        match element {
            Some(element) => Child::Single(element),
            None => Child::Empty,
        }
    }
}

impl<Window> From<Element<Window>> for Child<Window> {
    fn from(element: Element<Window>) -> Self {
        Child::Single(element)
    }
}

impl<Window, W: Widget<Window> + WidgetMeta + 'static> From<W> for Child<Window> {
    fn from(widget: W) -> Self {
        Child::Single(Element {
            widget: Box::new(widget),
            children: Box::new([])
        })
    }
}

impl<Window> fmt::Debug for dyn Widget<Window> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl<Window, Inner: Widget<Window> + 'static> Widget<Window> for WithKey<Inner> {
    #[inline]
    fn should_update(&self, widget: &dyn Widget<Window>, children: &[Element<Window>]) -> bool {
        self.inner.should_update(widget, children)
    }

    #[inline]
    fn render(&self, children: Box<[Element<Window>]>) -> Box<[Element<Window>]> {
        self.inner.render(children)
    }

    #[inline]
    fn mount(&mut self, parent_handle: &Window, rectangle: &Rectangle) -> Option<Window> {
        self.inner.mount(parent_handle, rectangle)
    }

    #[inline]
    fn unmount(&mut self, handle: &Window) {
        self.inner.unmount(handle)
    }

    #[inline]
    fn layout(
        &mut self,
        node_id: NodeId,
        box_constraints: BoxConstraints,
        response: Option<(NodeId, Size)>,
        fiber_tree: &FiberTree<Window>,
        layout_context: &mut LayoutContext,
    ) -> LayoutResult {
        self.inner.layout(node_id, box_constraints, response, fiber_tree, layout_context)
    }

    #[inline]
    fn paint(&mut self, handle: &Window, rectangle: &Rectangle, paint_context: &mut PaintContext<Window>) {
        self.inner.paint(handle, rectangle, paint_context)
    }
}

impl<Inner: WidgetMeta> WidgetMeta for WithKey<Inner> {
    #[inline]
    fn name(&self) -> &'static str {
        self.inner.name()
    }

    #[inline]
    fn key(&self) -> Option<Key> {
        Some(self.key)
    }

    #[inline]
    fn as_any(&self) -> &dyn any::Any {
        self.inner.as_any()
    }
}

#[inline]
pub fn same_widget<ConcreteWidget, Window>(
    this: &ConcreteWidget,
    other: &dyn Widget<Window>
) -> bool
where ConcreteWidget: Widget<Window> + Sized + PartialEq + 'static {
    compare_widget(this, other, PartialEq::eq)
}

#[inline]
pub fn compare_widget<ConcreteWidget, Window>(
    this: &ConcreteWidget,
    other: &dyn Widget<Window>,
    comparer: impl FnOnce(&ConcreteWidget, &ConcreteWidget) -> bool) -> bool
where ConcreteWidget: Widget<Window> + Sized + 'static {
    other
        .as_any()
        .downcast_ref::<ConcreteWidget>()
        .map(|other| comparer(this, other))
        .unwrap_or(false)
}
