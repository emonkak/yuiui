use yuiui_support::slot_tree::NodeId;

use crate::geometrics::Rectangle;
use crate::graphics::{Background, Color, Primitive};
use crate::widget::{DrawContext, ElementNode, Widget};

#[derive(Debug)]
pub struct Button {
    pub background: Background,
}

impl Widget for Button {
    type State = ();

    fn initial_state(&self) -> Self::State {
        Self::State::default()
    }

    fn draw(
        &self,
        bounds: Rectangle,
        _children: &[NodeId],
        _context: &mut DrawContext,
        _state: &mut Self::State,
    ) -> Primitive {
        Primitive::Quad {
            bounds,
            background: self.background,
            border_radius: 8.0,
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
        }
    }
}

impl From<Button> for ElementNode {
    fn from(widget: Button) -> Self {
        widget.into_boxed().into()
    }
}
