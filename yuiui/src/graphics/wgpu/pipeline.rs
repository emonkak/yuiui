use raw_window_handle::HasRawWindowHandle;
use std::mem;

use super::layer::Layer;
use super::quad::Quad;
use super::renderer::Renderer;
use super::text::Text;
use crate::geometrics::{Rectangle, Transform};
use crate::graphics::{Background, Primitive};
use crate::text::FontLoader;

#[derive(Debug)]
pub struct Pipeline {
    primary_layer: Layer,
    child_layers: Vec<Layer>,
}

impl Pipeline {
    pub fn new() -> Self {
        Self {
            primary_layer: Layer::new(None, Transform::IDENTITY),
            child_layers: Vec::new(),
        }
    }

    pub fn primary_layer(&self) -> &Layer {
        &self.primary_layer
    }

    pub fn child_layers(&self) -> &[Layer] {
        &self.child_layers
    }

    pub fn push<Window, FontLoader>(
        &mut self,
        primitive: Primitive,
        renderer: &mut Renderer<Window, FontLoader, FontLoader::Bundle, FontLoader::FontId>,
    ) where
        Window: HasRawWindowHandle,
        FontLoader: self::FontLoader,
    {
        match primitive {
            Primitive::None => {}
            Primitive::Batch(primitives) => {
                for primitive in primitives {
                    self.push(primitive, renderer);
                }
            }
            Primitive::Transform(transform, primitive) => {
                let standby_layer = self.switch_layer(self.primary_layer.bounds, transform);
                self.push(*primitive, renderer);
                if let Some(standby_layer) = standby_layer {
                    self.restore_layer(standby_layer);
                }
            }
            Primitive::Clip(clip_bounds, primitive) => {
                let bounds = match self.primary_layer.bounds {
                    Some(bounds) => bounds.intersection(clip_bounds).unwrap_or(Rectangle::ZERO),
                    None => clip_bounds,
                };
                let standby_layer = self.switch_layer(Some(bounds), self.primary_layer.transform);
                self.push(*primitive, renderer);
                if let Some(standby_layer) = standby_layer {
                    self.restore_layer(standby_layer);
                }
            }
            Primitive::Quad {
                bounds,
                background,
                border_radius,
                border_width,
                border_color,
            } => {
                self.primary_layer.quads.push(Quad {
                    position: [bounds.x, bounds.y],
                    size: [bounds.width, bounds.height],
                    color: match background {
                        Background::Color(color) => color.into_linear(),
                    },
                    border_radius,
                    border_width,
                    border_color: border_color.into_linear(),
                });
            }
            Primitive::Text {
                content,
                bounds,
                color,
                font,
                font_size,
                horizontal_align,
                vertical_align,
            } => {
                let segments = renderer.compute_segments(&content, font.clone());
                self.primary_layer.texts.push(Text {
                    content,
                    segments,
                    bounds,
                    color,
                    font_size,
                    horizontal_align,
                    vertical_align,
                })
            }
        }
    }

    fn switch_layer(&mut self, bounds: Option<Rectangle>, transform: Transform) -> Option<Layer> {
        if self.primary_layer.is_empty() {
            self.primary_layer.bounds = bounds;
            self.primary_layer.transform = transform;
            None
        } else {
            let layer = Layer::new(bounds, transform);
            Some(mem::replace(&mut self.primary_layer, layer))
        }
    }

    fn restore_layer(&mut self, standby_layer: Layer) {
        self.child_layers
            .push(mem::replace(&mut self.primary_layer, standby_layer));
    }
}
