use raw_window_handle::HasRawWindowHandle;
use std::mem;

use crate::geometrics::Rectangle;
use crate::graphics::{Background, Primitive, Transform};
use crate::text::FontLoader;

use super::quad::Quad;
use super::renderer::Renderer;
use super::text::Text;

#[derive(Debug)]
pub struct Pipeline {
    primary_layer: Layer,
    standby_layers: Vec<Layer>,
    prepared_layers: Vec<Layer>,
}

#[derive(Debug)]
pub struct Layer {
    depth: usize,
    pub bounds: Option<Rectangle>,
    pub transform: Transform,
    pub quads: Vec<Quad>,
    pub texts: Vec<Text>,
}

impl Pipeline {
    pub fn new() -> Self {
        Self {
            primary_layer: Layer::new(0, None, Transform::IDENTITY),
            standby_layers: Vec::new(),
            prepared_layers: Vec::new(),
        }
    }

    pub fn primary_layer(&self) -> &Layer {
        &self.primary_layer
    }

    pub fn prepared_layers(&self) -> &[Layer] {
        &self.prepared_layers
    }

    pub fn push<Window, FontLoader>(
        &mut self,
        primitive: &Primitive,
        depth: usize,
        renderer: &mut Renderer<Window, FontLoader, FontLoader::Bundle, FontLoader::FontId>,
    ) where
        Window: HasRawWindowHandle,
        FontLoader: self::FontLoader,
    {
        if self.primary_layer.depth >= depth {
            self.restore_layer();
        }
        self.process_primitive(primitive, depth, renderer);
    }

    pub fn finish(&mut self) {
        while self.restore_layer() {}
        debug_assert!(self.standby_layers.is_empty());
    }

    fn process_primitive<Window, FontLoader>(
        &mut self,
        primitive: &Primitive,
        depth: usize,
        renderer: &mut Renderer<Window, FontLoader, FontLoader::Bundle, FontLoader::FontId>,
    ) where
        Window: HasRawWindowHandle,
        FontLoader: self::FontLoader,
    {
        match primitive {
            Primitive::Batch(primitives) => {
                for primitive in primitives {
                    self.process_primitive(primitive, depth, renderer);
                }
            }
            Primitive::Transform(transform) => {
                self.switch_layer(
                    depth,
                    self.primary_layer.bounds,
                    self.primary_layer.transform * *transform,
                );
            }
            Primitive::Clip(clip_bounds) => {
                let bounds = match self.primary_layer.bounds {
                    Some(bounds) => bounds.intersection(clip_bounds).unwrap_or(Rectangle::ZERO),
                    None => *clip_bounds,
                };
                self.switch_layer(depth, Some(bounds), self.primary_layer.transform);
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
                    border_radius: *border_radius,
                    border_width: *border_width,
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
                let segments = renderer.compute_segments(content, font.clone());
                self.primary_layer.texts.push(Text {
                    content: content.clone(),
                    segments,
                    bounds: *bounds,
                    color: *color,
                    font_size: *font_size,
                    horizontal_align: *horizontal_align,
                    vertical_align: *vertical_align,
                })
            }
        }
    }

    fn switch_layer(&mut self, depth: usize, bounds: Option<Rectangle>, transform: Transform) {
        if self.primary_layer.is_empty() {
            debug_assert!(self.primary_layer.depth <= depth);
            self.primary_layer.bounds = bounds;
            self.primary_layer.transform = transform;
        } else {
            let layer = Layer::new(depth, bounds, transform);
            self.standby_layers
                .push(mem::replace(&mut self.primary_layer, layer));
        }
    }

    fn restore_layer(&mut self) -> bool {
        if let Some(standby_layer) = self.standby_layers.pop() {
            self.prepared_layers
                .push(mem::replace(&mut self.primary_layer, standby_layer));
            true
        } else {
            false
        }
    }
}

impl Layer {
    fn new(depth: usize, bounds: Option<Rectangle>, transform: Transform) -> Self {
        Self {
            depth,
            bounds,
            transform,
            quads: Vec::new(),
            texts: Vec::new(),
        }
    }

    fn is_empty(&self) -> bool {
        self.quads.is_empty() && self.texts.is_empty()
    }
}
