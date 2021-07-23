use std::any::Any;

use crate::tree::NodeId;
use crate::widget::element::{Children, Key};
use crate::widget::PolymophicWidget;

#[derive(Debug)]
pub struct RenderState<Handle> {
    pub rendered_children: Option<Children<Handle>>,
    pub deleted_children: Vec<NodeId>,
    pub state: Box<dyn Any>,
    pub dirty: bool,
    pub mounted: bool,
    pub key: Option<Key>,
}

impl<Handle> RenderState<Handle> {
    pub fn new(
        widget: &dyn PolymophicWidget<Handle>,
        children: Children<Handle>,
        key: Option<Key>,
    ) -> Self {
        let mut initial_state = widget.initial_state();
        let rendered_children = widget.render(children, &mut *initial_state).into();
        Self {
            rendered_children: Some(rendered_children),
            deleted_children: Vec::new(),
            state: initial_state,
            dirty: true,
            mounted: false,
            key,
        }
    }
}
