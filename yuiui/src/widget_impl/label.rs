use yuiui_support::slot_tree::NodeId;

use crate::geometrics::Rectangle;
use crate::graphics::{Color, Primitive};
use crate::text::{FontDescriptor, HorizontalAlign, VerticalAlign};
use crate::widget::{Attributes, DrawContext, ElementNode, Widget};

#[derive(Debug, PartialEq, Default)]
pub struct Label {
    pub content: String,
    pub color: Color,
    pub font: FontDescriptor,
    pub font_size: f32,
    pub horizontal_align: HorizontalAlign,
    pub vertical_align: VerticalAlign,
}

impl Widget for Label {
    type State = ();

    fn initial_state(&self) -> Self::State {
        Self::State::default()
    }

    fn should_update(
        &self,
        new_widget: &Self,
        old_attributes: &Attributes,
        new_attributes: &Attributes,
        _state: &Self::State,
    ) -> bool {
        self != new_widget || old_attributes != new_attributes
    }

    fn draw(
        &self,
        bounds: Rectangle,
        _children: &[NodeId],
        _context: &mut DrawContext,
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

impl From<Label> for ElementNode {
    fn from(widget: Label) -> Self {
        widget.into_boxed().into()
    }
}
