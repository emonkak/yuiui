use rust_ui_derive::WidgetMeta;

use crate::geometrics::Rectangle;
use crate::graphics::{Color, Primitive};
use crate::paint::PaintContext;
use crate::text::{FontDescriptor, HorizontalAlign, VerticalAlign};

use super::element::Children;
use super::widget::{Widget, WidgetMeta};

#[derive(PartialEq, WidgetMeta)]
pub struct Text {
    pub content: String,
    pub color: Color,
    pub font: FontDescriptor,
    pub font_size: f32,
    pub horizontal_align: HorizontalAlign,
    pub vertical_align: VerticalAlign,
}

impl<Renderer> Widget<Renderer> for Text {
    type State = ();

    fn should_update(
        &self,
        _children: &Children<Renderer>,
        _state: &Self::State,
        new_widget: &Self,
        _new_children: &Children<Renderer>,
    ) -> bool {
        self != new_widget
    }

    fn draw(
        &self,
        _children: &Children<Renderer>,
        _state: &mut Self::State,
        bounds: Rectangle,
        _renderer: &mut Renderer,
        _context: &mut PaintContext,
    ) -> Option<Primitive> {
        Primitive::Text {
            bounds,
            content: self.content.clone(),
            color: self.color,
            font: self.font.clone(),
            font_size: self.font_size,
            horizontal_align: self.horizontal_align,
            vertical_align: self.vertical_align,
        }
        .into()
    }
}
