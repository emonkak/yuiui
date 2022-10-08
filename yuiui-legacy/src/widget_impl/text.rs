use slot_vec::graph::NodeId;

use crate::geometrics::Rect;
use crate::graphics::{Color, Primitive};
use crate::text::{FontDescriptor, HorizontalAlign, VerticalAlign};
use crate::widget::{DrawContext, ElementInstance, Widget};

#[derive(Debug, PartialEq, Default)]
pub struct Text {
    pub content: String,
    pub color: Color,
    pub font: FontDescriptor,
    pub font_size: f32,
    pub horizontal_align: HorizontalAlign,
    pub vertical_align: VerticalAlign,
}

impl<State, Message> Widget<State, Message> for Text {
    type LocalState = ();

    fn initial_state(&self) -> Self::LocalState {
        ()
    }

    fn should_update(&self, new_widget: &Self) -> bool {
        self != new_widget
    }

    fn draw(
        &self,
        bounds: Rect,
        _children: &[NodeId],
        _context: &mut DrawContext<State, Message>,
        _state: &mut Self::LocalState,
    ) -> Primitive {
        Primitive::Text {
            bounds,
            content: self.content.clone(),
            color: self.color,
            font: self.font.clone(),
            font_size: self.font_size,
            horizontal_align: self.horizontal_align,
            vertical_align: self.vertical_align,
        }
    }
}

impl<State: 'static, Message: 'static> From<Text> for ElementInstance<State, Message> {
    fn from(widget: Text) -> Self {
        widget.into_rc().into()
    }
}
