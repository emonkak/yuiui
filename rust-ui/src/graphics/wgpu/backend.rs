use crate::base::Rectangle;
use crate::graphics::transformation::Transformation;
use crate::graphics::viewport::Viewport;

use super::draw_pipeline::DrawPipeline;
use super::quad;
use super::settings::Settings;

#[derive(Debug)]
pub struct Backend {
    quad_pipeline: quad::Pipeline,
}

impl Backend {
    pub fn new(device: &wgpu::Device, _settings: Settings, format: wgpu::TextureFormat) -> Self {
        let quad_pipeline = quad::Pipeline::new(device, format);

        Self { quad_pipeline }
    }

    pub fn draw(
        &mut self,
        device: &wgpu::Device,
        staging_belt: &mut wgpu::util::StagingBelt,
        encoder: &mut wgpu::CommandEncoder,
        frame: &wgpu::TextureView,
        draw_pipeline: &DrawPipeline,
        viewport: &Viewport,
    ) {
        let scale_factor = viewport.scale_factor() as f32;
        let transformation = viewport.projection();

        self.flush(
            device,
            staging_belt,
            encoder,
            &frame,
            &draw_pipeline,
            Rectangle::from(viewport.logical_size()),
            scale_factor,
            transformation,
        );

        for layer in &draw_pipeline.layers {
            self.flush(
                device,
                staging_belt,
                encoder,
                &frame,
                &layer.pipeline,
                layer.bounds,
                scale_factor,
                transformation,
            );
        }
    }

    fn flush(
        &mut self,
        device: &wgpu::Device,
        staging_belt: &mut wgpu::util::StagingBelt,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        draw_pipeline: &DrawPipeline,
        bounds: Rectangle,
        scale_factor: f32,
        transformation: Transformation,
    ) {
        let bounds = bounds.scale(scale_factor).snap();

        if !draw_pipeline.quads.is_empty() {
            self.quad_pipeline.draw(
                device,
                staging_belt,
                encoder,
                target,
                draw_pipeline.quads.as_slices(),
                bounds,
                scale_factor,
                transformation,
            );
        }
    }
}
