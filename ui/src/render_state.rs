use std::any::Any;

use geometrics::Rectangle;
use tree::NodeId;
use widget::{Element, WidgetDyn};

#[derive(Debug)]
pub struct RenderState<Handle> {
    pub rendered_children: Option<Box<[Element<Handle>]>>,
    pub deleted_children: Vec<NodeId>,
    pub state: Box<dyn Any>,
    pub rectangle: Rectangle,
    pub dirty: bool,
    pub mounted: bool,
}

impl<Handle> RenderState<Handle> {
    pub fn new(widget: &dyn WidgetDyn<Handle>, children: Box<[Element<Handle>]>) -> Self {
        let mut initial_state = widget.initial_state();
        let rendered_children = widget.render(children, &mut *initial_state);
        Self {
            rendered_children: Some(rendered_children),
            deleted_children: Vec::new(),
            state: initial_state,
            dirty: true,
            rectangle: Rectangle::ZERO,
            mounted: false,
        }
    }

    pub fn update(&mut self, widget: &dyn WidgetDyn<Handle>, children: Box<[Element<Handle>]>) {
        let rendered_children = widget.render(children, &mut *self.state);
        self.dirty = true;
        self.rendered_children = Some(rendered_children);
    }
}
