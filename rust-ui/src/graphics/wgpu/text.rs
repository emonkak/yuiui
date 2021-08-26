use wgpu_glyph::{ab_glyph, GlyphCruncher};

use crate::geometrics::{PhysicalRectangle, Rectangle, Size};
use crate::graphics::{Color, Transform};
use crate::text::{HorizontalAlign, VerticalAlign};

#[derive(Debug)]
pub struct Pipeline {
    draw_brush: wgpu_glyph::GlyphBrush<()>,
    measure_brush: glyph_brush::GlyphBrush<()>,
}

#[derive(Debug, Clone)]
pub struct Text {
    pub content: String,
    pub segments: Vec<Segment>,
    pub bounds: Rectangle,
    pub color: Color,
    pub font_size: f32,
    pub horizontal_align: HorizontalAlign,
    pub vertical_align: VerticalAlign,
}

#[derive(Clone, Copy, Debug)]
pub struct Segment {
    pub font_id: wgpu_glyph::FontId,
    pub start: usize,
    pub end: usize,
}

impl Pipeline {
    pub fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        default_font: ab_glyph::FontArc,
        multithreading: bool,
    ) -> Self {
        let draw_brush = wgpu_glyph::GlyphBrushBuilder::using_font(default_font.clone())
            .initial_cache_size((2048, 2048))
            .draw_cache_multithread(multithreading)
            .build(device, format);

        let measure_brush = glyph_brush::GlyphBrushBuilder::using_font(default_font).build();

        Self {
            draw_brush,
            measure_brush,
        }
    }

    pub fn run(
        &mut self,
        device: &wgpu::Device,
        staging_belt: &mut wgpu::util::StagingBelt,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        texts: &[Text],
        bounds: PhysicalRectangle,
        projection: Transform,
        transform: Transform,
        scale_factor: f32,
    ) {
        for text in texts {
            let section = text.create_section(scale_factor);
            self.draw_brush.queue(section);
        }

        self.draw_brush
            .draw_queued_with_transform_and_scissoring(
                device,
                staging_belt,
                encoder,
                target,
                (projection * transform).into(),
                wgpu_glyph::Region {
                    x: bounds.x,
                    y: bounds.y,
                    width: bounds.width,
                    height: bounds.height,
                },
            )
            .expect("Draw text");
    }

    pub fn add_font(
        &mut self,
        font_bytes: Vec<u8>,
    ) -> Result<wgpu_glyph::FontId, ab_glyph::InvalidFont> {
        let font = ab_glyph::FontArc::try_from_vec(font_bytes)?;
        let _ = self.measure_brush.add_font(font.clone());
        Ok(self.draw_brush.add_font(font))
    }

    pub fn measure(
        &mut self,
        content: &str,
        segments: Vec<Segment>,
        font_size: f32,
        size: Size,
    ) -> Size {
        let section = wgpu_glyph::Section {
            bounds: (size.width, size.height),
            text: segments
                .iter()
                .map(|segment| wgpu_glyph::Text {
                    text: &content[segment.start..segment.end],
                    scale: font_size.into(),
                    font_id: segment.font_id,
                    extra: wgpu_glyph::Extra::default(),
                })
                .collect(),
            ..Default::default()
        };

        if let Some(bounds) = self.measure_brush.glyph_bounds(section) {
            Size {
                width: bounds.width().ceil(),
                height: bounds.height().ceil(),
            }
        } else {
            Size::ZERO
        }
    }

    pub fn trim_measurement_cache(&mut self) {
        loop {
            let action = self.measure_brush.process_queued(|_, _| {}, |_| {});

            match action {
                Ok(_) => break,
                Err(glyph_brush::BrushError::TextureTooSmall { suggested }) => {
                    let (width, height) = suggested;
                    self.measure_brush.resize_texture(width, height);
                }
            }
        }
    }
}

impl Text {
    pub fn create_section<'a>(&'a self, scale_factor: f32) -> wgpu_glyph::Section<'a> {
        let scaled_bounds = self.bounds.scale(scale_factor);
        wgpu_glyph::Section {
            screen_position: (
                match self.horizontal_align {
                    HorizontalAlign::Left => scaled_bounds.x,
                    HorizontalAlign::Center => scaled_bounds.x + scaled_bounds.width / 2.0,
                    HorizontalAlign::Right => scaled_bounds.x + scaled_bounds.width,
                },
                match self.vertical_align {
                    VerticalAlign::Top => scaled_bounds.y,
                    VerticalAlign::Middle => scaled_bounds.y + scaled_bounds.height / 2.0,
                    VerticalAlign::Bottom => scaled_bounds.y + scaled_bounds.height,
                },
            ),
            bounds: (scaled_bounds.width, scaled_bounds.height),
            text: self
                .segments
                .iter()
                .map(|segment| self.create_text(segment, scale_factor))
                .collect(),
            layout: wgpu_glyph::Layout::default()
                .h_align(match self.horizontal_align {
                    HorizontalAlign::Left => wgpu_glyph::HorizontalAlign::Left,
                    HorizontalAlign::Center => wgpu_glyph::HorizontalAlign::Center,
                    HorizontalAlign::Right => wgpu_glyph::HorizontalAlign::Right,
                })
                .v_align(match self.vertical_align {
                    VerticalAlign::Top => wgpu_glyph::VerticalAlign::Top,
                    VerticalAlign::Middle => wgpu_glyph::VerticalAlign::Center,
                    VerticalAlign::Bottom => wgpu_glyph::VerticalAlign::Bottom,
                }),
        }
    }

    pub fn create_text(&self, segment: &Segment, scale_factor: f32) -> wgpu_glyph::Text {
        wgpu_glyph::Text {
            text: &self.content[segment.start..segment.end],
            scale: (self.font_size * scale_factor).into(),
            font_id: segment.font_id,
            extra: wgpu_glyph::Extra {
                color: self.color.into(),
                z: 0.0,
            },
        }
    }
}
