use futures::task::{FutureObj, Spawn};
use raw_window_handle::HasRawWindowHandle;
use std::io;

use crate::graphics::color::Color;
use crate::graphics::renderer::Renderer as RendererTrait;
use crate::graphics::viewport::Viewport;

use super::backend::Backend;
use super::draw_pipeline::DrawPipeline;
use super::settings::Settings;

pub struct Renderer {
    settings: Settings,
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    staging_belt: wgpu::util::StagingBelt,
    local_pool: futures::executor::LocalPool,
    format: wgpu::TextureFormat,
    backend: Backend,
}

impl Renderer {
    const CHUNK_SIZE: u64 = 10 * 1024;

    pub fn new<W: HasRawWindowHandle>(window: &W, settings: Settings) -> io::Result<Self> {
        futures::executor::block_on(Self::request(settings, window)).ok_or(io::Error::new(
            io::ErrorKind::NotFound,
            "A suitable graphics adapter or device could not be found",
        ))
    }

    pub async fn request<W: HasRawWindowHandle>(settings: Settings, window: &W) -> Option<Self> {
        let instance = wgpu::Instance::new(settings.internal_backend);

        let surface = unsafe { instance.create_surface(window) };

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: if settings.antialiasing.is_none() {
                    wgpu::PowerPreference::LowPower
                } else {
                    wgpu::PowerPreference::HighPerformance
                },
                compatible_surface: Some(&surface),
            })
            .await?;

        let format = adapter.get_swap_chain_preferred_format(&surface)?;

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
            .ok()?;

        let staging_belt = wgpu::util::StagingBelt::new(Self::CHUNK_SIZE);
        let local_pool = futures::executor::LocalPool::new();

        let backend = Backend::new(&device, settings, format);

        Some(Self {
            surface,
            settings,
            device,
            queue,
            staging_belt,
            local_pool,
            format,
            backend,
        })
    }
}

impl RendererTrait for Renderer {
    type DrawArea = wgpu::SwapChain;
    type DrawPipeline = self::DrawPipeline;

    fn create_draw_area(&mut self, viewport: &Viewport) -> Self::DrawArea {
        let physical_size = viewport.physical_size();
        self.device.create_swap_chain(
            &self.surface,
            &wgpu::SwapChainDescriptor {
                usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
                format: self.format,
                present_mode: self.settings.present_mode,
                width: physical_size.width,
                height: physical_size.height,
            },
        )
    }

    fn perform_draw(
        &mut self,
        draw_pipeline: &Self::DrawPipeline,
        draw_area: &mut Self::DrawArea,
        viewport: &Viewport,
        background_color: Color,
    ) {
        let frame = draw_area.get_current_frame().expect("Next frame");

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some(concat!(module_path!(), " encoder")),
            });

        let _ = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(concat!(module_path!(), " render pass")),
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: &frame.output.view,
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

        self.backend.draw(
            &mut self.device,
            &mut self.staging_belt,
            &mut encoder,
            &frame.output.view,
            draw_pipeline,
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
