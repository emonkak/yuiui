use yuiui_support::slot_tree::NodeId;

use crate::geometrics::Rectangle;
use crate::graphics::{Color, Primitive};
use crate::text::{FontDescriptor, HorizontalAlign, VerticalAlign};
use crate::widget::{DrawContext, ElementNode, Widget};

#[derive(Debug, PartialEq, Default)]
pub struct Label {
    pub content: String,
    pub color: Color,
    pub font: FontDescriptor,
    pub font_size: f32,
    pub horizontal_align: HorizontalAlign,
    pub vertical_align: VerticalAlign,
}

impl<Message> Widget<Message> for Label {
    type State = ();

    fn initial_state(&self) -> Self::State {
        Self::State::default()
    }

    fn should_update(
        &self,
        new_widget: &Self,
        _state: &Self::State,
    ) -> bool {
        self != new_widget
    }

    fn draw(
        &self,
        bounds: Rectangle,
        _children: &[NodeId],
        _context: &mut DrawContext<Message>,
        _state: &mut Self::State,
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

impl<Message: 'static> From<Label> for ElementNode<Message> {
    fn from(widget: Label) -> Self {
        widget.into_boxed().into()
    }
}
