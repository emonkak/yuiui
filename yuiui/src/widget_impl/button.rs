use yuiui_support::slot_tree::NodeId;

use crate::geometrics::Rectangle;
use crate::graphics::{Background, Color, Primitive};
use crate::widget::{DrawContext, ElementNode, Widget};

#[derive(Debug)]
pub struct Button {
    pub background: Background,
}

impl<Message> Widget<Message> for Button {
    type State = ();

    fn initial_state(&self) -> Self::State {
        Self::State::default()
    }

    fn draw(
        &self,
        bounds: Rectangle,
        _children: &[NodeId],
        _context: &mut DrawContext<Message>,
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

impl<Message: 'static> From<Button> for ElementNode<Message> {
    fn from(widget: Button) -> Self {
        widget.into_rc().into()
    }
}
