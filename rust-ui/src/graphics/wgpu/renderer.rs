use futures::task::{FutureObj, Spawn};
use raw_window_handle::HasRawWindowHandle;

use crate::base::Rectangle;
use crate::graphics::background::Background;
use crate::graphics::color::Color;
use crate::graphics::renderer::{Pipeline as PipelineTrait, Renderer as RendererTrait};
use crate::graphics::viewport::Viewport;

use super::backend::Backend;
use super::quad::Quad;
use super::settings::Settings;

pub struct Renderer {
    settings: Settings,
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    format: wgpu::TextureFormat,
    staging_belt: wgpu::util::StagingBelt,
    local_pool: futures::executor::LocalPool,
    backend: Backend,
}

#[derive(Debug)]
pub enum RequstError {
    AdapterNotFound,
    TextureFormatNotFound,
    RequestDeviceError(wgpu::RequestDeviceError),
}

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

pub enum Primitive {
    None,
    Batch(Vec<Primitive>),
    Quad {
        bounds: Rectangle,
        background: Background,
        border_radius: f32,
        border_width: f32,
        border_color: Color,
    },
}

impl Renderer {
    const CHUNK_SIZE: u64 = 10 * 1024;

    pub fn new<W: HasRawWindowHandle>(window: &W, settings: Settings) -> Result<Self, RequstError> {
        futures::executor::block_on(Self::request(window, settings))
    }

    pub async fn request<W: HasRawWindowHandle>(
        window: &W,
        settings: Settings,
    ) -> Result<Self, RequstError> {
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
            .await
            .ok_or(RequstError::AdapterNotFound)?;

        let format = adapter
            .get_swap_chain_preferred_format(&surface)
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
        let backend = Backend::new(&device, settings, format);

        Ok(Self {
            settings,
            surface,
            device,
            queue,
            format,
            staging_belt,
            local_pool,
            backend,
        })
    }
}

impl RendererTrait for Renderer {
    type DrawArea = wgpu::SwapChain;
    type Primitive = self::Primitive;
    type Pipeline = self::Pipeline;

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

    fn create_pipeline(&mut self, viewport: &Viewport) -> Self::Pipeline {
        let bounds = Rectangle::from(viewport.logical_size());
        Pipeline {
            primary_layer: Layer {
                quads: Vec::new(),
                bounds,
            },
            layers: Vec::new(),
        }
    }

    fn perform_pipeline(
        &mut self,
        swap_chain: &mut Self::DrawArea,
        pipeline: &Self::Pipeline,
        viewport: &Viewport,
        background_color: Color,
    ) {
        let frame = swap_chain.get_current_frame().expect("Next frame");

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

impl PipelineTrait<Primitive> for Pipeline {
    fn push(&mut self, primitive: &Primitive) {
        match primitive {
            Primitive::None => {}
            Primitive::Batch(primitives) => {
                for primitive in primitives {
                    self.push(primitive);
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
                    border_radius: *border_radius,
                    border_width: *border_width,
                    border_color: border_color.into_linear(),
                });
            }
        }
    }
}

impl Default for Primitive {
    fn default() -> Self {
        Primitive::None
    }
}
