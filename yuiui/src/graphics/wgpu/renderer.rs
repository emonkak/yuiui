use futures::task::{FutureObj, Spawn};
use raw_window_handle::HasRawWindowHandle;
use std::collections::HashMap;
use std::io;
use wgpu_glyph::ab_glyph;

use crate::geometrics::{Size, Transform, Viewport};
use crate::graphics::{Color, Primitive};
use crate::text::{FontDescriptor, FontLoader};
use super::layer::Layer;
use super::{quad, text, Pipeline, Settings};

pub struct Renderer<Window, FontLoader, FontBundle, FontId> {
    settings: Settings,
    instance: wgpu::Instance,
    device: wgpu::Device,
    queue: wgpu::Queue,
    format: wgpu::TextureFormat,
    staging_belt: wgpu::util::StagingBelt,
    local_pool: futures::executor::LocalPool,
    window: Window,
    font_loader: FontLoader,
    font_bundle_map: HashMap<FontDescriptor, Option<FontBundle>>,
    draw_font_map: HashMap<FontId, Option<wgpu_glyph::FontId>>,
    quad_pipeline: quad::Pipeline,
    text_pipeline: text::Pipeline,
}

#[derive(Debug)]
pub enum RequstError {
    AdapterNotFound,
    TextureFormatNotFound,
    RequestDeviceFailed(wgpu::RequestDeviceError),
    DefaultFontNotFound,
    FontLoadingFailed(io::Error),
    InvalidFont(ab_glyph::InvalidFont),
}

impl<Window, FontLoader> Renderer<Window, FontLoader, FontLoader::Bundle, FontLoader::FontId>
where
    Window: HasRawWindowHandle,
    FontLoader: self::FontLoader,
{
    const CHUNK_SIZE: u64 = 10 * 1024;

    pub fn new(
        window: Window,
        font_loader: FontLoader,
        settings: Settings,
    ) -> Result<Self, RequstError> {
        futures::executor::block_on(Self::request(window, font_loader, settings))
    }

    pub async fn request(
        window: Window,
        mut font_loader: FontLoader,
        settings: Settings,
    ) -> Result<Self, RequstError> {
        let instance = wgpu::Instance::new(settings.internal_backend);

        let compatible_surface = unsafe { instance.create_surface(&window) };

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: settings.power_preference,
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
            .map_err(RequstError::RequestDeviceFailed)?;

        let staging_belt = wgpu::util::StagingBelt::new(Self::CHUNK_SIZE);
        let local_pool = futures::executor::LocalPool::new();
        let mut font_bundle_map = HashMap::new();

        let default_font = {
            let default_bundle = font_loader
                .load_bundle(&settings.default_font)
                .ok_or(RequstError::DefaultFontNotFound)?;
            let primary_font = font_loader.get_primary_font(&default_bundle);
            let font_bytes = font_loader
                .load_font(primary_font)
                .map_err(RequstError::FontLoadingFailed)?;
            font_bundle_map.insert(settings.default_font.clone(), Some(default_bundle));
            ab_glyph::FontArc::try_from_vec(font_bytes).map_err(RequstError::InvalidFont)?
        };

        let quad_pipeline = quad::Pipeline::new(&device, format);
        let text_pipeline =
            text::Pipeline::new(&device, format, default_font, settings.text_multithreading);

        Ok(Self {
            settings,
            instance,
            device,
            queue,
            format,
            staging_belt,
            local_pool,
            window,
            font_loader,
            font_bundle_map,
            draw_font_map: HashMap::new(),
            quad_pipeline,
            text_pipeline,
        })
    }

    pub fn measure_text(
        &mut self,
        content: &str,
        segments: Vec<text::Segment>,
        font_size: f32,
        size: Size,
    ) -> Size {
        self.text_pipeline
            .measure(content, segments, font_size, size)
    }

    pub fn compute_segments(
        &mut self,
        content: &str,
        font_descriptor: FontDescriptor,
    ) -> Vec<text::Segment> {
        let font_loader = &mut self.font_loader;
        let text_pipeline = &mut self.text_pipeline;

        let bundle = self
            .font_bundle_map
            .entry(font_descriptor)
            .or_insert_with_key(|font_descriptor| font_loader.load_bundle(font_descriptor))
            .as_ref();

        match bundle {
            None => {
                vec![text::Segment {
                    font_id: wgpu_glyph::FontId(0),
                    start: 0,
                    end: content.len(),
                }]
            }
            Some(bundle) => {
                let mut segments = Vec::new();
                for (loader_font_id, range) in font_loader.split_segments(bundle, content) {
                    let font_id = self
                        .draw_font_map
                        .entry(loader_font_id)
                        .or_insert_with(|| {
                            if let Ok(font_bytes) = font_loader.load_font(loader_font_id) {
                                text_pipeline.add_font(font_bytes).ok()
                            } else {
                                None
                            }
                        })
                        .unwrap_or(wgpu_glyph::FontId(0));
                    segments.push(text::Segment {
                        font_id,
                        start: range.start,
                        end: range.end,
                    });
                }
                segments
            }
        }
    }

    fn flush_pipeline(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        pipeline: &Pipeline,
        viewport: &Viewport,
    ) {
        let projection = viewport.projection();
        let scale_factor = viewport.scale_factor();

        self.flush_layer(
            encoder,
            &target,
            pipeline.primary_layer(),
            projection,
            scale_factor,
        );

        for child_layer in pipeline.child_layers() {
            self.flush_layer(encoder, &target, &child_layer, projection, scale_factor);
        }
    }

    fn flush_layer(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        layer: &Layer,
        projection: Transform,
        scale_factor: f32,
    ) {
        let scissor_bounds = layer.bounds.map(|bounds| bounds.scale(scale_factor).snap());

        if !layer.quads.is_empty() {
            self.quad_pipeline.run(
                &self.device,
                &mut self.staging_belt,
                encoder,
                target,
                &layer.quads,
                scissor_bounds,
                projection,
                layer.transform,
                scale_factor,
            );
        }

        if !layer.texts.is_empty() {
            self.text_pipeline.run(
                &self.device,
                &mut self.staging_belt,
                encoder,
                target,
                &layer.texts,
                scissor_bounds,
                projection,
                layer.transform,
                scale_factor,
            );
        }
    }
}

impl<Window, FontLoader> crate::graphics::Renderer
    for Renderer<Window, FontLoader, FontLoader::Bundle, FontLoader::FontId>
where
    Window: HasRawWindowHandle,
    FontLoader: self::FontLoader,
{
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

    fn create_pipeline(&mut self, primitive: Primitive) -> Self::Pipeline {
        let mut pipeline = Pipeline::new();
        pipeline.push(primitive, self);
        // FIXME: Is this really necessary?
        self.text_pipeline.trim_measurement_cache();
        pipeline
    }

    fn perform_pipeline(
        &mut self,
        pipeline: &mut Self::Pipeline,
        surface: &mut Self::Surface,
        viewport: &Viewport,
        background_color: Color,
    ) {
        let frame = surface.get_current_frame().expect("Next frame");

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some(concat!(module_path!(), " encoder")),
            });

        let view = frame
            .output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

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

        self.flush_pipeline(&mut encoder, &view, pipeline, viewport);

        self.staging_belt.finish();
        self.queue.submit(Some(encoder.finish()));

        self.local_pool
            .spawner()
            .spawn_obj(FutureObj::from(Box::new(self.staging_belt.recall())))
            .expect("Recall staging belt");

        self.local_pool.run_until_stalled();
    }
}
