use std::any;
use std::array;
use std::fmt;
use std::mem;

use geometrics::{Rectangle, Size};
use layout::{BoxConstraints, LayoutResult, LayoutContext};
use paint::PaintContext;
use tree::{Node, NodeId, Tree};

pub type FiberTree<Window> = Tree<Fiber<Window>>;

pub type FiberNode<Window> = Node<Fiber<Window>>;

#[derive(Debug)]
pub struct Fiber<Window> {
    pub(crate) widget: Box<dyn WidgetDyn<Window>>,
    pub(crate) rendered_children: Option<Box<[Element<Window>]>>,
    pub(crate) handle: Option<Window>,
    pub(crate) state: Option<Box<dyn any::Any>>,
    pub(crate) dirty: bool,
    pub(crate) mounted: bool,
}

pub struct Element<Window> {
    pub widget: Box<dyn WidgetDyn<Window>>,
    pub children: Box<[Element<Window>]>,
}

#[derive(Debug)]
pub enum Child<Window> {
    Multiple(Vec<Element<Window>>),
    Single(Element<Window>),
    Empty,
}

pub trait Widget<Window>: WidgetMeta {
    type State;

    fn initial_state(&self) -> Self::State;

    #[inline(always)]
    fn should_update(&self, _next_widget: &Self, _next_children: &[Element<Window>]) -> bool {
        true
    }

    #[inline(always)]
    fn will_update(&self, _next_widget: &Self, _next_children: &[Element<Window>]) {
    }

    #[inline(always)]
    fn did_update(&self, _prev_widget: &Self) {
    }

    #[inline(always)]
    fn render(&self, children: Box<[Element<Window>]>, _state: &mut Self::State) -> Box<[Element<Window>]> {
        children
    }

    #[inline(always)]
    fn mount(&mut self, _parent_handle: &Window, _rectangle: &Rectangle) -> Option<Window> {
        None
    }

    #[inline(always)]
    fn unmount(&mut self, _handle: &Window) {
    }

    #[inline(always)]
    fn layout(
        &self,
        node_id: NodeId,
        box_constraints: BoxConstraints,
        response: Option<(NodeId, Size)>,
        tree: &FiberTree<Window>,
        _layout_context: &mut LayoutContext,
        _state: &mut Self::State
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

    #[inline(always)]
    fn paint(&self, _rectangle: &Rectangle, _handle: &Window, _paint_context: &mut PaintContext<Window>) {
    }
}

pub trait WidgetDyn<Window>: WidgetMeta {
    fn initial_state(&self) -> Box<dyn any::Any>;

    fn should_update(&self, next_widget: &dyn WidgetDyn<Window>, next_children: &[Element<Window>]) -> bool;

    fn will_update(&self, next_widget: &dyn WidgetDyn<Window>, next_children: &[Element<Window>]);

    fn did_update(&self, prev_widget: &dyn WidgetDyn<Window>);

    fn render(&self, children: Box<[Element<Window>]>, state: &mut dyn any::Any) -> Box<[Element<Window>]>;

    fn mount(&mut self, parent_handle: &Window, rectangle: &Rectangle) -> Option<Window>;

    fn unmount(&mut self, handle: &Window);

    fn layout(&self, node_id: NodeId, box_constraints: BoxConstraints, response: Option<(NodeId, Size)>, tree: &FiberTree<Window>, layout_context: &mut LayoutContext, state: &mut dyn any::Any) -> LayoutResult;

    fn paint(&self, rectangle: &Rectangle, handle: &Window, paint_context: &mut PaintContext<Window>);
}

pub trait WidgetMeta {
    #[inline(always)]
    fn name(&self) -> &'static str {
        let full_name = any::type_name::<Self>();
        full_name
            .rsplit_once("::")
            .map(|(_, last)| last)
            .unwrap_or(full_name)
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

    fn as_any(&self) -> &dyn any::Any;
}

pub struct WithKey<T> {
    inner: T,
    key: Key,
}

pub type Key = usize;

impl<Window> Fiber<Window> {
    pub(crate) fn new(element: Element<Window>) -> Fiber<Window> {
        let mut initial_state = element.widget.initial_state();
        let rendered_children = element.widget.render(element.children, &mut *initial_state);
        Fiber {
            widget: element.widget,
            rendered_children: Some(rendered_children),
            handle: None,
            state: None,
            dirty: true,
            mounted: false,
        }
    }

    pub fn as_widget<T: Widget<Window> + 'static>(&self) -> Option<&T> {
        self.widget.as_any().downcast_ref()
    }

    pub(crate) fn update(&mut self, element: Element<Window>) -> bool {
        if self.widget.should_update(&*element.widget, &element.children) {
            self.widget.will_update(&*element.widget, &element.children);

            let prev_widget = mem::replace(&mut self.widget, element.widget);
            let mut state = self.state.take().unwrap_or_else(|| self.widget.initial_state());
            let rendered_children = self.widget.render(element.children, &mut *state);

            self.dirty = true;
            self.state = Some(state);
            self.rendered_children = Some(rendered_children);

            self.widget.did_update(&*prev_widget);
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

    pub(crate) fn paint<'a>(&'a mut self, rectangle: &Rectangle, parent_handle: &'a Window, paint_context: &mut PaintContext<Window>) -> &'a Window {
        let widget = &mut self.widget;

        if !self.mounted {
            self.handle = widget.mount(&parent_handle, rectangle);
            self.mounted = true;
        }

        let handle = self.handle.as_ref().unwrap_or(parent_handle);
        widget.paint(rectangle, handle, paint_context);

        handle
    }
}

impl<Window> fmt::Display for Fiber<Window> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.widget.name())
    }
}

impl<Window> Element<Window> {
    pub fn new<State: 'static, const N: usize>(widget: impl Widget<Window, State=State> + 'static, children: [Element<Window>; N]) -> Self {
        Self {
            widget: Box::new(widget),
            children: Box::new(children),
        }
    }

    pub fn build<State: 'static, const N: usize>(widget: impl Widget<Window, State=State> + 'static, children: [Child<Window>; N]) -> Self {
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

impl<Window, State: 'static, W: Widget<Window, State=State> + WidgetMeta + 'static> From<W> for Child<Window> {
    fn from(widget: W) -> Self {
        Child::Single(Element {
            widget: Box::new(widget),
            children: Box::new([])
        })
    }
}

impl<Window> fmt::Debug for dyn WidgetDyn<Window> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl<Window, State: 'static, T: Widget<Window, State=State> + WidgetMeta + 'static> WidgetDyn<Window> for T {
    #[inline(always)]
    fn initial_state(&self) -> Box<dyn any::Any> {
        Box::new(self.initial_state())
    }

    #[inline(always)]
    fn should_update(&self, next_widget: &dyn WidgetDyn<Window>, next_children: &[Element<Window>]) -> bool {
        next_widget
            .as_any()
            .downcast_ref::<Self>()
            .map(|next_widget| self.should_update(next_widget, next_children))
            .unwrap_or(true)
    }

    #[inline(always)]
    fn will_update(&self, next_widget: &dyn WidgetDyn<Window>, next_children: &[Element<Window>]) {
        if let Some(next_widget) = next_widget.as_any().downcast_ref() {
            self.will_update(next_widget, next_children)
        }
    }

    #[inline(always)]
    fn did_update(&self, next_widget: &dyn WidgetDyn<Window>) {
        if let Some(next_widget) = next_widget.as_any().downcast_ref() {
            self.did_update(next_widget)
        }
    }

    #[inline(always)]
    fn render(&self, children: Box<[Element<Window>]>, state: &mut dyn any::Any) -> Box<[Element<Window>]> {
        self.render(children, state.downcast_mut().unwrap())
    }

    #[inline(always)]
    fn mount(&mut self, parent_handle: &Window, rectangle: &Rectangle) -> Option<Window> {
        self.mount(parent_handle, rectangle)
    }

    #[inline(always)]
    fn unmount(&mut self, handle: &Window) {
        self.unmount(handle)
    }

    #[inline(always)]
    fn layout(
        &self,
        node_id: NodeId,
        box_constraints: BoxConstraints,
        response: Option<(NodeId, Size)>,
        tree: &FiberTree<Window>,
        layout_context: &mut LayoutContext,
        state: &mut dyn any::Any
    ) -> LayoutResult {
        self.layout(
            node_id,
            box_constraints,
            response,
            tree,
            layout_context,
            state.downcast_mut::<State>().unwrap()
        )
    }

    #[inline(always)]
    fn paint(&self, rectangle: &Rectangle, handle: &Window, paint_context: &mut PaintContext<Window>) {
        self.paint(rectangle, handle, paint_context)
    }
}

impl<Window, T: Widget<Window> + 'static> Widget<Window> for WithKey<T> {
    type State = T::State;

    #[inline(always)]
    fn initial_state(&self) -> Self::State {
        self.inner.initial_state()
    }

    #[inline(always)]
    fn should_update(&self, next_widget: &Self, next_children: &[Element<Window>]) -> bool {
        self.inner.should_update(&next_widget.inner, next_children)
    }

    #[inline(always)]
    fn will_update(&self, next_widget: &Self, next_children: &[Element<Window>]) {
        self.inner.will_update(&next_widget.inner, next_children)
    }

    #[inline(always)]
    fn did_update(&self, prev_widget: &Self) {
        self.inner.did_update(&prev_widget.inner)
    }

    #[inline(always)]
    fn render(&self, children: Box<[Element<Window>]>, state: &mut Self::State) -> Box<[Element<Window>]> {
        self.inner.render(children, state)
    }

    #[inline(always)]
    fn mount(&mut self, parent_handle: &Window, rectangle: &Rectangle) -> Option<Window> {
        self.inner.mount(parent_handle, rectangle)
    }

    #[inline(always)]
    fn unmount(&mut self, handle: &Window) {
        self.inner.unmount(handle)
    }

    #[inline(always)]
    fn layout(
        &self,
        node_id: NodeId,
        box_constraints: BoxConstraints,
        response: Option<(NodeId, Size)>,
        tree: &FiberTree<Window>,
        layout_context: &mut LayoutContext,
        state: &mut Self::State
    ) -> LayoutResult {
        self.inner.layout(node_id, box_constraints, response, tree, layout_context, state)
    }

    #[inline(always)]
    fn paint(&self, rectangle: &Rectangle, handle: &Window, paint_context: &mut PaintContext<Window>) {
        self.inner.paint(rectangle, handle, paint_context)
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
    fn as_any(&self) -> &dyn any::Any {
        self.inner.as_any()
    }
}
