use rust_ui_derive::WidgetMeta;

use crate::geometrics::Rectangle;
use crate::graphics::{Color, Primitive};
use crate::paint::PaintContext;
use crate::text::{FontDescriptor, HorizontalAlign, VerticalAlign};

use super::element::Children;
use super::state::StateCell;
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
        new_widget: &Self,
        _old_children: &Children<Renderer>,
        _new_children: &Children<Renderer>,
        _state: StateCell<Self::State>,
    ) -> bool {
        self != new_widget
    }

    fn draw(
        &self,
        bounds: Rectangle,
        _state: StateCell<Self::State>,
        _renderer: &mut Renderer,
        _context: &mut PaintContext<Renderer>,
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
