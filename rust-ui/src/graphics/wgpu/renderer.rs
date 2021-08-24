use futures::task::{FutureObj, Spawn};
use raw_window_handle::HasRawWindowHandle;

use crate::geometrics::Rectangle;
use crate::graphics::{Color, Transformation, Viewport};

use super::pipeline::{Layer, Pipeline};
use super::quad;
use super::settings::Settings;

pub struct Renderer<Window> {
    settings: Settings,
    instance: wgpu::Instance,
    device: wgpu::Device,
    queue: wgpu::Queue,
    format: wgpu::TextureFormat,
    staging_belt: wgpu::util::StagingBelt,
    local_pool: futures::executor::LocalPool,
    backend: Backend,
    window: Window,
}

#[derive(Debug)]
pub enum RequstError {
    AdapterNotFound,
    TextureFormatNotFound,
    RequestDeviceError(wgpu::RequestDeviceError),
}

#[derive(Debug)]
struct Backend {
    quad_pipeline: quad::Pipeline,
}

impl<Window: HasRawWindowHandle> Renderer<Window> {
    const CHUNK_SIZE: u64 = 10 * 1024;

    pub fn new(window: Window, settings: Settings) -> Result<Self, RequstError> {
        futures::executor::block_on(Self::request(window, settings))
    }

    pub async fn request(
        window: Window,
        settings: Settings,
    ) -> Result<Self, RequstError> {
        let instance = wgpu::Instance::new(settings.internal_backend);

        let compatible_surface = unsafe { instance.create_surface(&window) };

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: if settings.antialiasing.is_none() {
                    wgpu::PowerPreference::LowPower
                } else {
                    wgpu::PowerPreference::HighPerformance
                },
                compatible_surface: Some(&compatible_surface),
            })
            .await
            .ok_or(RequstError::AdapterNotFound)?;

        let format = compatible_surface
            .get_preferred_format(&adapter)
            .ok_or(RequstError::TextureFormatNotFound)?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some(concat!(module_path!(), " device descriptor")),
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits {
                        max_bind_groups: 2,
                        ..wgpu::Limits::default()
                    },
                },
                None,
            )
            .await
            .map_err(RequstError::RequestDeviceError)?;

        let staging_belt = wgpu::util::StagingBelt::new(Self::CHUNK_SIZE);
        let local_pool = futures::executor::LocalPool::new();
        let backend = Backend::new(&device, format, &settings);

        Ok(Self {
            settings,
            instance,
            device,
            queue,
            format,
            staging_belt,
            local_pool,
            backend,
            window,
        })
    }
}

impl<Window: HasRawWindowHandle> crate::graphics::Renderer for Renderer<Window> {
    type Surface = wgpu::Surface;
    type Pipeline = Pipeline;

    fn create_surface(&mut self, viewport: &Viewport) -> Self::Surface {
        let mut surface = unsafe { self.instance.create_surface(&self.window) };
        self.configure_surface(&mut surface, viewport);
        surface
    }

    fn configure_surface(&mut self, surface: &mut Self::Surface, viewport: &Viewport) {
        let physical_size = viewport.physical_size();
        surface.configure(
            &self.device,
            &wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: self.format,
                present_mode: self.settings.present_mode,
                width: physical_size.width,
                height: physical_size.height,
            },
        )
    }

    fn create_pipeline(&mut self, viewport: &Viewport) -> Self::Pipeline {
        let bounds = Rectangle::from(viewport.logical_size());
        Pipeline::new(bounds)
    }

    fn perform_pipeline(
        &mut self,
        surface: &mut Self::Surface,
        pipeline: &mut Self::Pipeline,
        viewport: &Viewport,
        background_color: Color,
    ) {
        let frame = surface.get_current_frame().expect("Next frame");

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some(concat!(module_path!(), " encoder")),
            });

        let view = frame.output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let _ = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(concat!(module_path!(), " render pass")),
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear({
                        let [r, g, b, a] = background_color.into_linear();

                        wgpu::Color {
                            r: f64::from(r),
                            g: f64::from(g),
                            b: f64::from(b),
                            a: f64::from(a),
                        }
                    }),
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });

        self.backend.run(
            &mut self.device,
            &mut self.staging_belt,
            &mut encoder,
            &view,
            pipeline,
            viewport,
        );

        self.staging_belt.finish();
        self.queue.submit(Some(encoder.finish()));

        self.local_pool
            .spawner()
            .spawn_obj(FutureObj::from(Box::new(self.staging_belt.recall())))
            .expect("Recall staging belt");

        self.local_pool.run_until_stalled();
    }
}

impl Backend {
    fn new(device: &wgpu::Device, format: wgpu::TextureFormat, _settings: &Settings) -> Self {
        let quad_pipeline = quad::Pipeline::new(device, format);
        Self { quad_pipeline }
    }

    fn run(
        &mut self,
        device: &wgpu::Device,
        staging_belt: &mut wgpu::util::StagingBelt,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        pipeline: &Pipeline,
        viewport: &Viewport,
    ) {
        let scale_factor = viewport.scale_factor() as f32;
        let transformation = viewport.projection();

        self.flush(
            device,
            staging_belt,
            encoder,
            &target,
            pipeline.primary_layer(),
            Rectangle::from(viewport.logical_size()),
            scale_factor,
            transformation,
        );

        for layer in pipeline.finished_layers() {
            self.flush(
                device,
                staging_belt,
                encoder,
                &target,
                &layer,
                layer.bounds(),
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
        layer: &Layer,
        bounds: Rectangle,
        scale_factor: f32,
        transformation: Transformation,
    ) {
        let bounds = bounds.scale(scale_factor).snap();

        if !layer.quads().is_empty() {
            self.quad_pipeline.run(
                device,
                staging_belt,
                encoder,
                target,
                &layer.quads(),
                bounds,
                scale_factor,
                transformation,
            );
        }
    }
}
