use std::mem;

use crate::geometrics::{Rectangle, Vector};
use crate::graphics::{Background, Primitive};

use super::quad::Quad;

#[derive(Debug)]
pub struct Pipeline {
    primary_layer: Layer,
    standby_layers: Vec<Layer>,
    finished_layers: Vec<Layer>,
    translations: Vec<(usize, Vector)>,
}

#[derive(Debug)]
pub struct Layer {
    depth: usize,
    bounds: Rectangle,
    quads: Vec<Quad>,
}

impl Pipeline {
    pub fn new(bounds: Rectangle) -> Self {
        Self {
            primary_layer: Layer::new(0, bounds),
            standby_layers: Vec::new(),
            finished_layers: Vec::new(),
            translations: Vec::new(),
        }
    }

    pub fn primary_layer(&self) -> &Layer {
        &self.primary_layer
    }

    pub fn finished_layers(&self) -> &[Layer] {
        &self.finished_layers
    }

    fn switch_layer(&mut self, layer: Layer) {
        self.standby_layers
            .push(mem::replace(&mut self.primary_layer, layer));
    }

    fn restore_layer(&mut self) -> bool {
        if let Some(standby_layer) = self.standby_layers.pop() {
            self.finished_layers
                .push(mem::replace(&mut self.primary_layer, standby_layer));
            true
        } else {
            false
        }
    }

    fn get_translation(&mut self, depth: usize) -> Vector {
        let mut vector = Vector::ZERO;

        for i in 0..self.translations.len() {
            let (trans_depth, trans_vector) = &self.translations[i];
            if *trans_depth <= depth {
                self.translations.drain(i..);
                break;
            }
            vector = vector + *trans_vector;
        }

        vector
    }

    fn process_primitive(&mut self, primitive: &Primitive, depth: usize, translation: &mut Vector) {
        match primitive {
            Primitive::Batch(primitives) => {
                for primitive in primitives {
                    self.process_primitive(primitive, depth, translation);
                }
            }
            Primitive::Translate(vector) => {
                *translation = *translation + *vector;
                self.translations.push((depth, *vector));
            }
            Primitive::Clip(bounds) => {
                self.switch_layer(Layer::new(depth, *bounds));
            }
            Primitive::Quad {
                bounds,
                background,
                border_radius,
                border_width,
                border_color,
            } => {
                let translated_bounds = bounds.translate(*translation);
                self.primary_layer.quads.push(Quad {
                    position: [translated_bounds.x, translated_bounds.y],
                    size: [translated_bounds.width, translated_bounds.height],
                    color: match background {
                        Background::Color(color) => color.into_linear(),
                    },
                    border_radius: *border_radius,
                    border_width: *border_width,
                    border_color: border_color.into_linear(),
                });
            }
        }
    }
}

impl crate::graphics::Pipeline for Pipeline {
    fn push(&mut self, primitive: &Primitive, depth: usize) {
        let mut translation = self.get_translation(depth);
        if self.primary_layer.depth <= depth {
            self.restore_layer();
        }
        self.process_primitive(primitive, depth, &mut translation);
    }

    fn finish(&mut self) {
        while self.restore_layer() {}
        debug_assert!(self.standby_layers.is_empty());
    }
}

impl Layer {
    fn new(depth: usize, bounds: Rectangle) -> Self {
        Self {
            depth,
            bounds,
            quads: Vec::new(),
        }
    }

    pub fn bounds(&self) -> Rectangle {
        self.bounds
    }

    pub fn quads(&self) -> &[Quad] {
        &self.quads
    }
}
