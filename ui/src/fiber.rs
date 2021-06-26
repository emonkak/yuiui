use std::fmt;

use geometrics::{Point, Rectangle, Size};
use tree::{DisplayTreeData, Node, NodeId, Tree};
use widget::widget::{Widget, Element};

pub type RenderingTree<Window> = Tree<Fiber<Window>>;

pub type RenderingNode<Window> = Node<Fiber<Window>>;

#[derive(Debug)]
pub struct Fiber<Window> {
    pub(crate) widget: Box<dyn Widget<Window>>,
    pub(crate) rendered_children: Option<Box<[Element<Window>]>>,
    pub(crate) handle: Option<Window>,
    pub(crate) rectangle: Rectangle,
    pub(crate) dirty: bool,
}

impl<Window> Fiber<Window> {
    pub(crate) fn new(element: Element<Window>) -> Fiber<Window> {
        let rendered_children = element.instance.render_children(element.children);
        Fiber {
            widget: element.instance,
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

    pub fn as_widget<T: Widget<Window> + 'static>(&self) -> Option<&T> {
        self.widget.as_any().downcast_ref()
    }

    pub(crate) fn update(&mut self, element: Element<Window>) {
        let rendered_children = element.instance.render_children(element.children);
        self.widget = element.instance;
        self.dirty = true;
        self.rendered_children = Some(rendered_children);
    }

    pub(crate) fn should_update(&mut self, element: &Element<Window>) -> bool {
        self.widget.should_update(&element)
    }

    pub(crate) fn disconnect(&mut self) {
        let widget = &mut self.widget;
        if let Some(handle) = self.handle.as_ref() {
            widget.disconnect(handle);
        }
    }
}

impl<Window> DisplayTreeData for Fiber<Window> {
    fn fmt_start(&self, f: &mut fmt::Formatter, node_id: NodeId) -> fmt::Result {
        write!(
            f,
            "<{} id=\"{}\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\">",
            self.widget,
            node_id,
            self.rectangle.point.x,
            self.rectangle.point.y,
            self.rectangle.size.width,
            self.rectangle.size.height
        )
    }

    fn fmt_end(&self, f: &mut fmt::Formatter, _node_id: NodeId) -> fmt::Result {
        write!(f, "</{}>", self.widget)
    }
}
