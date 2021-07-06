use std::any;
use std::array;
use std::fmt;
use std::mem;

use geometrics::{Rectangle, Size};
use layout::{BoxConstraints, LayoutResult, LayoutContext};
use paint::PaintContext;
use tree::{Link, NodeId, Tree};

pub type FiberTree<Handle> = Tree<Fiber<Handle>>;

pub type FiberNode<Handle> = Link<Fiber<Handle>>;

#[derive(Debug)]
pub struct Fiber<Handle> {
    pub(crate) widget: Box<dyn WidgetDyn<Handle>>,
    pub(crate) rendered_children: Option<Box<[Element<Handle>]>>,
    pub(crate) handle: Option<Handle>,
    pub(crate) state: Option<Box<dyn any::Any>>,
    pub(crate) dirty: bool,
    pub(crate) mounted: bool,
}

pub struct Element<Handle> {
    pub widget: Box<dyn WidgetDyn<Handle>>,
    pub children: Box<[Element<Handle>]>,
}

#[derive(Debug)]
pub enum Child<Handle> {
    Multiple(Vec<Element<Handle>>),
    Single(Element<Handle>),
    Empty,
}

pub trait Widget<Handle>: WidgetMeta {
    type State;

    fn initial_state(&self) -> Self::State;

    #[inline(always)]
    fn should_update(&self, _next_widget: &Self, _next_children: &[Element<Handle>]) -> bool {
        true
    }

    #[inline(always)]
    fn will_update(&self, _next_widget: &Self, _next_children: &[Element<Handle>]) {
    }

    #[inline(always)]
    fn did_update(&self, _prev_widget: &Self) {
    }

    #[inline(always)]
    fn render(&self, children: Box<[Element<Handle>]>, _state: &mut Self::State) -> Box<[Element<Handle>]> {
        children
    }

    #[inline(always)]
    fn mount(&mut self, _parent_handle: &Handle, _rectangle: &Rectangle) -> Option<Handle> {
        None
    }

    #[inline(always)]
    fn unmount(&mut self, _handle: &Handle) {
    }

    #[inline(always)]
    fn layout(
        &self,
        node_id: NodeId,
        box_constraints: BoxConstraints,
        response: Option<(NodeId, Size)>,
        tree: &FiberTree<Handle>,
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
    fn paint(&self, _rectangle: &Rectangle, _handle: &Handle, _paint_context: &mut PaintContext<Handle>) {
    }
}

pub trait WidgetDyn<Handle>: WidgetMeta {
    fn initial_state(&self) -> Box<dyn any::Any>;

    fn should_update(&self, next_widget: &dyn WidgetDyn<Handle>, next_children: &[Element<Handle>]) -> bool;

    fn will_update(&self, next_widget: &dyn WidgetDyn<Handle>, next_children: &[Element<Handle>]);

    fn did_update(&self, prev_widget: &dyn WidgetDyn<Handle>);

    fn render(&self, children: Box<[Element<Handle>]>, state: &mut dyn any::Any) -> Box<[Element<Handle>]>;

    fn mount(&mut self, parent_handle: &Handle, rectangle: &Rectangle) -> Option<Handle>;

    fn unmount(&mut self, handle: &Handle);

    fn layout(&self, node_id: NodeId, box_constraints: BoxConstraints, response: Option<(NodeId, Size)>, tree: &FiberTree<Handle>, layout_context: &mut LayoutContext, state: &mut dyn any::Any) -> LayoutResult;

    fn paint(&self, rectangle: &Rectangle, handle: &Handle, paint_context: &mut PaintContext<Handle>);
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

impl<Handle> Fiber<Handle> {
    pub(crate) fn new(widget: Box<dyn WidgetDyn<Handle>>, children: Box<[Element<Handle>]>) -> Fiber<Handle> {
        let mut initial_state = widget.initial_state();
        let rendered_children = widget.render(children, &mut *initial_state);
        Fiber {
            widget,
            rendered_children: Some(rendered_children),
            handle: None,
            state: None,
            dirty: true,
            mounted: false,
        }
    }

    pub fn as_widget<T: Widget<Handle> + 'static>(&self) -> Option<&T> {
        self.widget.as_any().downcast_ref()
    }

    pub(crate) fn update(&mut self, element: Element<Handle>) -> bool {
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

    pub(crate) fn paint<'a>(&'a mut self, rectangle: &Rectangle, parent_handle: &'a Handle, paint_context: &mut PaintContext<Handle>) -> &'a Handle {
        let widget = &mut self.widget;

        if !self.mounted {
            self.handle = widget.mount(&parent_handle, rectangle);
            self.mounted = true;
        }

        self.dirty = false;

        let handle = self.handle.as_ref().unwrap_or(parent_handle);
        widget.paint(rectangle, handle, paint_context);

        handle
    }
}

impl<Handle> From<Element<Handle>> for Fiber<Handle> {
    fn from(element: Element<Handle>) -> Fiber<Handle> {
        Fiber::new(element.widget, element.children)
    }
}

impl<Handle> fmt::Display for Fiber<Handle> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.widget.name())
    }
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

impl<Handle> From<Vec<Element<Handle>>> for Child<Handle> {
    fn from(elements: Vec<Element<Handle>>) -> Self {
        Child::Multiple(elements)
    }
}

impl<Handle> From<Option<Element<Handle>>> for Child<Handle> {
    fn from(element: Option<Element<Handle>>) -> Self {
        match element {
            Some(element) => Child::Single(element),
            None => Child::Empty,
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

impl<Handle> fmt::Debug for dyn WidgetDyn<Handle> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl<Handle, State: 'static, T: Widget<Handle, State=State> + WidgetMeta + 'static> WidgetDyn<Handle> for T {
    #[inline(always)]
    fn initial_state(&self) -> Box<dyn any::Any> {
        Box::new(self.initial_state())
    }

    #[inline(always)]
    fn should_update(&self, next_widget: &dyn WidgetDyn<Handle>, next_children: &[Element<Handle>]) -> bool {
        next_widget
            .as_any()
            .downcast_ref::<Self>()
            .map(|next_widget| self.should_update(next_widget, next_children))
            .unwrap_or(true)
    }

    #[inline(always)]
    fn will_update(&self, next_widget: &dyn WidgetDyn<Handle>, next_children: &[Element<Handle>]) {
        if let Some(next_widget) = next_widget.as_any().downcast_ref() {
            self.will_update(next_widget, next_children)
        }
    }

    #[inline(always)]
    fn did_update(&self, next_widget: &dyn WidgetDyn<Handle>) {
        if let Some(next_widget) = next_widget.as_any().downcast_ref() {
            self.did_update(next_widget)
        }
    }

    #[inline(always)]
    fn render(&self, children: Box<[Element<Handle>]>, state: &mut dyn any::Any) -> Box<[Element<Handle>]> {
        self.render(children, state.downcast_mut().unwrap())
    }

    #[inline(always)]
    fn mount(&mut self, parent_handle: &Handle, rectangle: &Rectangle) -> Option<Handle> {
        self.mount(parent_handle, rectangle)
    }

    #[inline(always)]
    fn unmount(&mut self, handle: &Handle) {
        self.unmount(handle)
    }

    #[inline(always)]
    fn layout(
        &self,
        node_id: NodeId,
        box_constraints: BoxConstraints,
        response: Option<(NodeId, Size)>,
        tree: &FiberTree<Handle>,
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
    fn paint(&self, rectangle: &Rectangle, handle: &Handle, paint_context: &mut PaintContext<Handle>) {
        self.paint(rectangle, handle, paint_context)
    }
}

impl<Handle, T: Widget<Handle> + 'static> Widget<Handle> for WithKey<T> {
    type State = T::State;

    #[inline(always)]
    fn initial_state(&self) -> Self::State {
        self.inner.initial_state()
    }

    #[inline(always)]
    fn should_update(&self, next_widget: &Self, next_children: &[Element<Handle>]) -> bool {
        self.inner.should_update(&next_widget.inner, next_children)
    }

    #[inline(always)]
    fn will_update(&self, next_widget: &Self, next_children: &[Element<Handle>]) {
        self.inner.will_update(&next_widget.inner, next_children)
    }

    #[inline(always)]
    fn did_update(&self, prev_widget: &Self) {
        self.inner.did_update(&prev_widget.inner)
    }

    #[inline(always)]
    fn render(&self, children: Box<[Element<Handle>]>, state: &mut Self::State) -> Box<[Element<Handle>]> {
        self.inner.render(children, state)
    }

    #[inline(always)]
    fn mount(&mut self, parent_handle: &Handle, rectangle: &Rectangle) -> Option<Handle> {
        self.inner.mount(parent_handle, rectangle)
    }

    #[inline(always)]
    fn unmount(&mut self, handle: &Handle) {
        self.inner.unmount(handle)
    }

    #[inline(always)]
    fn layout(
        &self,
        node_id: NodeId,
        box_constraints: BoxConstraints,
        response: Option<(NodeId, Size)>,
        tree: &FiberTree<Handle>,
        layout_context: &mut LayoutContext,
        state: &mut Self::State
    ) -> LayoutResult {
        self.inner.layout(node_id, box_constraints, response, tree, layout_context, state)
    }

    #[inline(always)]
    fn paint(&self, rectangle: &Rectangle, handle: &Handle, paint_context: &mut PaintContext<Handle>) {
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
