use std::any::Any;
use std::any;

use geometrics::{Point, Rectangle, Size};
use layout::{BoxConstraints, LayoutResult};
use tree::{DebugTreeData, Node, NodeId, Tree, TreeFormatter};

pub mod fill;
pub mod flex;
pub mod null;
pub mod padding;

pub struct Null;

pub struct WidgetPod<WindowHandle, PaintContext> {
    pub(crate) widget: Box<dyn Widget<WindowHandle, PaintContext>>,
    pub(crate) rendered_children: Option<Box<[Element<WindowHandle, PaintContext>]>>,
    pub(crate) handle: Option<WindowHandle>,
    pub(crate) rectangle: Rectangle,
    pub(crate) dirty: bool,
}

pub trait Widget<WindowHandle, PaintContext> {
    fn name(&self) -> &'static str {
        let full_name = any::type_name::<Self>();
        full_name
            .rsplit_once("::")
            .map(|(_, last)| last)
            .unwrap_or(full_name)
    }

    fn layout(
        &mut self,
        node_id: NodeId,
        response: Option<(NodeId, Size)>,
        box_constraints: &BoxConstraints,
        rendering_tree: &mut RenderingTree<WindowHandle, PaintContext>,
    ) -> LayoutResult {
        if let Some((child, size)) = response {
            rendering_tree[child].arrange(Default::default());
            LayoutResult::Size(size)
        } else {
            if let Some(child) = rendering_tree[node_id].first_child() {
                LayoutResult::RequestChild(child, *box_constraints)
            } else {
                LayoutResult::Size(box_constraints.max)
            }
        }
    }

    fn connect(&mut self, _parent_handle: &WindowHandle, _rectangle: &Rectangle, _paint_context: &mut PaintContext) -> Option<WindowHandle> {
        None
    }

    fn disconnect(&mut self, _handle: &WindowHandle) {
    }

    fn paint(&mut self, _handle: &WindowHandle, _rectangle: &Rectangle, _paint_context: &mut PaintContext) {
    }

    fn render_children(&self, children: Box<[Element<WindowHandle, PaintContext>]>) -> Box<[Element<WindowHandle, PaintContext>]> {
        children
    }

    fn should_rerender(&self, _next_widget: &Box<dyn Widget<WindowHandle, PaintContext>>, _next_children: &Box<[Element<WindowHandle, PaintContext>]>) -> bool {
        true
    }

    fn same_widget(&self, other: &Box<dyn Widget<WindowHandle, PaintContext>>) -> bool where Self: Sized + PartialEq + 'static {
        other
            .as_any()
            .downcast_ref::<Self>()
            .map(|other| self == other)
            .unwrap_or(false)
    }

    fn as_any(&self) -> &dyn Any;
}

pub struct Element<WindowHandle, PaintContext> {
    pub widget: Box<dyn Widget<WindowHandle, PaintContext>>,
    pub children: Box<[Element<WindowHandle, PaintContext>]>,
}

pub type RenderingTree<WindowHandle, PaintContext> = Tree<WidgetPod<WindowHandle, PaintContext>>;

pub type RenderingNode<WindowHandle, PaintContext> = Node<WidgetPod<WindowHandle, PaintContext>>;

impl<WindowHandle, PaintContext> WidgetPod<WindowHandle, PaintContext> {
    pub(crate) fn new(element: Element<WindowHandle, PaintContext>) -> WidgetPod<WindowHandle, PaintContext> {
        let rendered_children = element.widget.render_children(element.children);
        WidgetPod {
            widget: element.widget,
            rendered_children: Some(rendered_children),
            handle: None,
            dirty: true,
            rectangle: Default::default(),
        }
    }

    pub fn arrange(&mut self, point: Point) {
        self.rectangle.point = point;
    }

    pub fn resize(&mut self, size: Size) {
        if self.rectangle.size != size {
            self.rectangle.size = size;
            self.dirty = true;
        }
    }

    pub fn as_widget<T: Widget<WindowHandle, PaintContext> + 'static>(&self) -> Option<&T> {
        self.widget.as_any().downcast_ref()
    }

    pub(crate) fn update(&mut self, element: Element<WindowHandle, PaintContext>) {
        let rendered_children = element.widget.render_children(element.children);
        self.widget = element.widget;
        self.dirty = true;
        self.rendered_children = Some(rendered_children);
    }

    pub(crate) fn should_update(&mut self, element: &Element<WindowHandle, PaintContext>) -> bool {
        self.widget.should_rerender(&element.widget, &element.children)
    }

    pub(crate) fn disconnect(&mut self) {
        let widget = &mut self.widget;
        if let Some(handle) = self.handle.as_ref() {
            widget.disconnect(handle);
        }
    }
}

impl<WindowHandle, PaintContext> DebugTreeData for WidgetPod<WindowHandle, PaintContext> {
    fn format(&self, node_id: NodeId, formatter: &mut TreeFormatter) {
        formatter
            .begin(self.widget.name())
            .push_attribute("id", node_id.to_string())
            .push_attribute("x", self.rectangle.point.x.to_string())
            .push_attribute("y", self.rectangle.point.y.to_string())
            .push_attribute("width", self.rectangle.size.width.to_string())
            .push_attribute("height", self.rectangle.size.height.to_string())
            .push_empty_attribute("dirty", self.dirty)
            .end();
    }
}

impl<WindowHandle, PaintContext> Element<WindowHandle, PaintContext> {
    fn to_string_rec(&self, level: usize) -> String {
        let name = self.widget.name();
        let indent_str = unsafe { String::from_utf8_unchecked(vec![b'\t'; level]) };
        if self.children.len() > 0 {
            let mut children_str = "".to_string();
            for i in 0..self.children.len() {
                children_str.push('\n');
                children_str.push_str(&self.children[i].to_string_rec(level + 1));
            }
            format!("{}<{}>{}\n{}</{}>", indent_str, name, children_str, indent_str, name)
        } else {
            format!("{}<{}></{}>", indent_str, name, name)
        }
    }
}

impl<WindowHandle, PaintContext> ToString for Element<WindowHandle, PaintContext> {
    fn to_string(&self) -> String {
        self.to_string_rec(0)
    }
}

pub fn el<WindowHandle, PaintContext, const N: usize>(widget: impl Widget<WindowHandle, PaintContext> + 'static, children: [Element<WindowHandle, PaintContext>; N]) -> Element<WindowHandle, PaintContext> {
    Element {
        widget: Box::new(widget),
        children: Box::new(children),
    }
}
