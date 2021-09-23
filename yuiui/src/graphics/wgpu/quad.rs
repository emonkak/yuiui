use bytemuck::{Pod, Zeroable};
use std::mem;

use crate::geometrics::{PhysicalRectangle, Transform};

#[derive(Debug)]
pub struct Pipeline {
    pipeline: wgpu::RenderPipeline,
    constants: wgpu::BindGroup,
    constants_buffer: wgpu::Buffer,
    instances: wgpu::Buffer,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
pub struct Quad {
    pub position: [f32; 2],
    pub size: [f32; 2],
    pub color: [f32; 4],
    pub border_color: [f32; 4],
    pub border_radius: f32,
    pub border_width: f32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
struct Uniforms {
    projection: [f32; 16],
    transform: [f32; 16],
    scale_factor: f32,
    _padding: [f32; 3],
}

const MAX_INSTANCES: usize = 100_000;

impl Pipeline {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Pipeline {
        let constant_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(concat!(module_path!(), " uniforms layout")),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(
                        mem::size_of::<Uniforms>() as wgpu::BufferAddress
                    ),
                },
                count: None,
            }],
        });

        let constants_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(concat!(module_path!(), " uniforms buffer")),
            size: mem::size_of::<Uniforms>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let constants = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(concat!(module_path!(), " uniforms bind group")),
            layout: &constant_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: constants_buffer.as_entire_binding(),
            }],
        });

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(concat!(module_path!(), " pipeline layout")),
            push_constant_ranges: &[],
            bind_group_layouts: &[&constant_layout],
        });

        let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some(concat!(module_path!(), " shader module")),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
                "shader/quad.wgsl"
            ))),
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(concat!(module_path!(), " pipeline")),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: mem::size_of::<Quad>() as u64,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &wgpu::vertex_attr_array!(
                        0 => Float32x2,
                        1 => Float32x2,
                        2 => Float32x4,
                        3 => Float32x4,
                        4 => Float32,
                        5 => Float32,
                    ),
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                }],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                front_face: wgpu::FrontFace::Cw,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
        });

        let instances = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(concat!(module_path!(), " instance buffer")),
            size: mem::size_of::<Quad>() as u64 * MAX_INSTANCES as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Pipeline {
            pipeline,
            constants,
            constants_buffer,
            instances,
        }
    }

    pub fn run(
        &mut self,
        device: &wgpu::Device,
        staging_belt: &mut wgpu::util::StagingBelt,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        instances: &[Quad],
        scissor_bounds: Option<PhysicalRectangle>,
        projection: Transform,
        transform: Transform,
        scale_factor: f32,
    ) {
        {
            let uniforms = Uniforms::new(projection, transform, scale_factor);
            let mut constants_buffer = staging_belt.write_buffer(
                encoder,
                &self.constants_buffer,
                0,
                wgpu::BufferSize::new(mem::size_of::<Uniforms>() as u64).unwrap(),
                device,
            );
            constants_buffer.copy_from_slice(bytemuck::bytes_of(&uniforms));
        }

        for i in (0..instances.len()).step_by(MAX_INSTANCES) {
            let end = (i + MAX_INSTANCES).min(instances.len());
            let count = end - i;

            {
                let instance_bytes = bytemuck::cast_slice(&instances[i..end]);
                let mut instance_buffer = staging_belt.write_buffer(
                    encoder,
                    &self.instances,
                    0,
                    wgpu::BufferSize::new(instance_bytes.len() as u64).unwrap(),
                    device,
                );
                instance_buffer.copy_from_slice(instance_bytes);
            }

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some(concat!(module_path!(), " render pass")),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: target,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &self.constants, &[]);
            render_pass.set_vertex_buffer(0, self.instances.slice(..));

            if let Some(bounds) = scissor_bounds {
                render_pass.set_scissor_rect(bounds.x, bounds.y, bounds.width, bounds.height);
            }

            render_pass.draw(0..4, 0..count as u32);
        }
    }
}

impl Uniforms {
    fn new(projection: Transform, transform: Transform, scale_factor: f32) -> Uniforms {
        Self {
            projection: projection.into(),
            transform: transform.into(),
            scale_factor,
            _padding: [0.0; 3],
        }
    }
}
