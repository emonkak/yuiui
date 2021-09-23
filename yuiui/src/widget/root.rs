use yuiui_support::slot_tree::NodeId;

use super::{ElementNode, LayoutContext, Widget, WidgetExt};
use crate::geometrics::{BoxConstraints, Size, Viewport};

#[derive(Debug)]
pub struct Root {
    pub initial_viewport: Viewport
}

pub struct State {
    viewport: Viewport,
}

impl Root {
    pub fn new(viewport: Viewport) -> Self {
        Self {
            initial_viewport: viewport,
        }
    }
}

impl Widget for Root {
    type State = State;

    fn initial_state(&self) -> Self::State {
        State {
            viewport: self.initial_viewport.clone()
        }
    }

    fn layout(
        &self,
        _box_constraints: BoxConstraints,
        children: &[NodeId],
        context: &mut LayoutContext,
        state: &mut Self::State,
    ) -> Size {
        let box_constraints = BoxConstraints::tight(state.viewport.logical_size());
        if let Some(child) = children.first() {
            context.layout_child(*child, box_constraints);
        }
        state.viewport.logical_size()
    }
}

impl From<Root> for ElementNode {
    fn from(widget: Root) -> Self {
        widget.into_boxed().into()
    }
}
