use std::mem;

use crate::base::Rectangle;
use crate::graphics::background::Background;
use crate::graphics::color::Color;

use super::quad::Quad;

#[derive(Debug, Clone)]
pub struct Pipeline {
    pub(crate) primary_layer: Layer,
    pub(crate) layers: Vec<Layer>,
}

#[derive(Debug, Clone)]
pub struct Layer {
    pub(crate) quads: Vec<Quad>,
    pub(crate) bounds: Rectangle,
}

impl Pipeline {
    pub fn push_quad(
        &mut self,
        bounds: Rectangle,
        background: Background,
        border_radius: f32,
        border_width: f32,
        border_color: Color,
    ) {
        self.primary_layer.quads.push(Quad {
            position: [bounds.x, bounds.y],
            size: [bounds.width, bounds.height],
            color: match background {
                Background::Color(color) => color.into_linear(),
            },
            border_radius: border_radius,
            border_width: border_width,
            border_color: border_color.into_linear(),
        });
    }

    pub fn push_layer(&mut self, bounds: Rectangle) {
        self.layers
            .push(mem::replace(&mut self.primary_layer, Layer::new(bounds)));
    }

    pub fn pop_layer(&mut self) {
        if let Some(layer) = self.layers.pop() {
            self.layers
                .push(mem::replace(&mut self.primary_layer, layer))
        }
    }
}

impl Layer {
    fn new(bounds: Rectangle) -> Self {
        Self {
            bounds,
            quads: Vec::new(),
        }
    }
}
