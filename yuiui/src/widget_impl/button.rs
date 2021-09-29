use yuiui_support::slot_tree::NodeId;

use crate::geometrics::Rectangle;
use crate::graphics::{Background, Color, Primitive};
use crate::widget::{DrawContext, ElementInstance, Widget};

#[derive(Debug)]
pub struct Button {
    pub background: Background,
}

impl<State, Message> Widget<State, Message> for Button {
    type LocalState = ();

    fn initial_state(&self) -> Self::LocalState {
        ()
    }

    fn draw(
        &self,
        bounds: Rectangle,
        _children: &[NodeId],
        _context: &mut DrawContext<State, Message>,
        _state: &mut Self::LocalState,
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

impl<State: 'static, Message: 'static> From<Button> for ElementInstance<State, Message> {
    fn from(widget: Button) -> Self {
        widget.into_rc().into()
    }
}
