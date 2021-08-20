use crate::graphics::transformation::Transformation;
use crate::graphics::viewport::Viewport;

use super::draw_op::DrawOp;
use super::layer::Layer;
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
        viewport: &Viewport,
        draw_op: &DrawOp,
    ) {
        let target_size = viewport.physical_size();
        let scale_factor = viewport.scale_factor() as f32;
        let transformation = viewport.projection();

        let layers = Layer::generate(draw_op, viewport);

        for layer in layers {
            self.flush(
                device,
                scale_factor,
                transformation,
                &layer,
                staging_belt,
                encoder,
                &frame,
                target_size.width,
                target_size.height,
            );
        }
    }

    fn flush(
        &mut self,
        device: &wgpu::Device,
        scale_factor: f32,
        transformation: Transformation,
        layer: &Layer,
        staging_belt: &mut wgpu::util::StagingBelt,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        _target_width: u32,
        _target_height: u32,
    ) {
        let bounds = layer.bounds.scale(scale_factor).snap();

        if !layer.quads.is_empty() {
            self.quad_pipeline.draw(
                device,
                staging_belt,
                encoder,
                &layer.quads,
                transformation,
                scale_factor,
                bounds,
                target,
            );
        }
    }
}
