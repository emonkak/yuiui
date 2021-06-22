use std::cell::RefCell;

use geometrics::{BoxConstraints, Rectangle, Point, Size};
use graph::{Graph, NodeId};
use widget::Widget;
use window::{WindowHandle, WindowHandler};

pub struct UIMain<WindowHandle, PaintContext> {
    state: RefCell<UIState<WindowHandle, PaintContext>>,
}

pub struct UIState<WindowHandle, PaintContext> {
    graph: Graph,
    widgets: Vec<Box<dyn Widget<WindowHandle, PaintContext>>>,
    widget_states: Vec<WidgetState<WindowHandle>>,
    root_handle: WindowHandle,
    layout_context: LayoutContext,
}

pub struct WidgetState<WindowHandle> {
    handle: Option<WindowHandle>,
}

pub struct LayoutContext {
    rectangles: Vec<Rectangle>
}

pub enum LayoutResult {
    Size(Size),
    RequestChild(NodeId, BoxConstraints),
}

impl<WindowHandle, PaintContext> UIMain<WindowHandle, PaintContext> {
    pub fn new(state: UIState<WindowHandle, PaintContext>) -> UIMain<WindowHandle, PaintContext> {
        UIMain {
            state: RefCell::new(state)
        }
    }
}

impl<WindowHandle, PaintContext> UIState<WindowHandle, PaintContext> {
    pub fn new(root_handle: WindowHandle) -> UIState<WindowHandle, PaintContext> {
        UIState {
            widgets: Vec::new(),
            widget_states: Vec::new(),
            graph: Default::default(),
            root_handle,
            layout_context: LayoutContext {
                rectangles: Vec::new(),
            }
        }
    }

    pub fn set_root(&mut self, root: NodeId) {
        self.graph.root = root;
    }
}

impl<WindowHandle: self::WindowHandle + 'static, PaintContext: 'static> WindowHandler<WindowHandle, PaintContext> for UIMain<WindowHandle, PaintContext> {
    fn connect(&self, handle: &WindowHandle) {
        let mut state = self.state.borrow_mut();
        state.root_handle = handle.clone();
    }

    fn paint(&self, paint_context: &mut PaintContext) {
        let mut state = self.state.borrow_mut();
        let root = state.graph.root;
        let size = state.root_handle.get_size();
        let box_constraints = BoxConstraints::tight(&size);
        state.layout(&box_constraints, root);
        state.paint(paint_context, root);
    }

    fn destroy(&self) {
        let state = self.state.borrow();
        state.root_handle.close();
    }
}

impl<WindowHandle: Clone, PaintContext> UIState<WindowHandle, PaintContext> {
    pub fn add<W>(&mut self, widget: W, children: &[NodeId]) -> NodeId
        where W: Widget<WindowHandle, PaintContext> + 'static
    {
        let node_id = self.graph.alloc_node();
        if node_id < self.widgets.len() {
            self.widgets[node_id] = Box::new(widget);
            self.widget_states[node_id] = WidgetState {
                handle: None
            };
            self.layout_context.rectangles[node_id] = Default::default();
        } else {
            self.widgets.push(Box::new(widget));
            self.widget_states.push(WidgetState {
                handle: None
            });
            self.layout_context.rectangles.push(Default::default());
        }
        for &child in children {
            self.graph.append_child(node_id, child);
        }
        node_id
    }

    fn paint(&mut self, paint_context: &mut PaintContext, root: NodeId) {
        fn paint_rec<WindowHandle: Clone, PaintContext>(
            graph: &Graph,
            widgets: &mut [Box<dyn Widget<WindowHandle, PaintContext>>],
            widget_states: &mut [WidgetState<WindowHandle>],
            layout_context: &LayoutContext,
            paint_context: &mut PaintContext,
            parent_handle: &WindowHandle,
            node: NodeId,
            point: Point
        ) {
            let widget = &mut widgets[node];
            let mut rectangle = layout_context.rectangles[node];
            rectangle.point = rectangle.point.offset(point);

            let handle = widget_states[node].handle.get_or_insert_with(|| {
                widget.connect(parent_handle, &rectangle, paint_context)
            }).clone();

            widget.paint(&handle, &rectangle, paint_context);

            for &child in &graph.children[node] {
                paint_rec(
                    graph,
                    widgets,
                    widget_states,
                    layout_context,
                    paint_context,
                    &handle,
                    child,
                    rectangle.point
                );
            }
        }

        paint_rec(
            &self.graph,
            &mut self.widgets,
            &mut self.widget_states,
            &self.layout_context,
            paint_context,
            &self.root_handle,
            root,
            Point { x: 0., y: 0. }
        );
    }

    fn layout(&mut self, box_constraints: &BoxConstraints, root: NodeId) {
        fn layout_rec<WindowHandle, PaintContext>(
            graph: &Graph,
            widgets: &mut [Box<dyn Widget<WindowHandle, PaintContext>>],
            layout_context: &mut LayoutContext,
            box_constraints: &BoxConstraints,
            node: NodeId
        ) -> Size {
            let mut size = None;
            loop {
                let widget = &mut widgets[node];
                let layout_res = widget.layout(
                    box_constraints,
                    &graph.children[node],
                    size,
                    layout_context
                );

                match layout_res {
                    LayoutResult::Size(size) => {
                        layout_context.resize_child(node, size);
                        return size;
                    }
                    LayoutResult::RequestChild(child, child_box_constraints) => {
                        size = Some(layout_rec(
                            graph,
                            widgets,
                            layout_context,
                            &child_box_constraints,
                            child
                        ));
                    }
                }
            }
        }

        layout_rec(
            &self.graph,
            &mut self.widgets,
            &mut self.layout_context,
            box_constraints,
            root
        );
    }
}

impl LayoutContext {
    pub fn position_child(&mut self, child: NodeId, point: Point) {
        self.rectangles[child].point = point;
    }

    pub fn resize_child(&mut self, child: NodeId, size: Size) {
        self.rectangles[child].size = size;
    }

    pub fn get_child_size(&self, child: NodeId) -> Size {
        self.rectangles[child].size
    }
}
