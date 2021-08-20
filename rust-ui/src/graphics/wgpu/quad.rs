use bytemuck::{Pod, Zeroable};
use std::mem;
use std::ops::Range;
use wgpu::util::DeviceExt;

use crate::base::PhysicalRectangle;
use crate::graphics::transformation::Transformation;

const QUAD_INDICES: [u16; 6] = [0, 1, 2, 0, 2, 3];

const QUAD_VERTS: [Vertex; 4] = [
    Vertex {
        _position: [0.0, 0.0],
    },
    Vertex {
        _position: [1.0, 0.0],
    },
    Vertex {
        _position: [1.0, 1.0],
    },
    Vertex {
        _position: [0.0, 1.0],
    },
];

const MAX_INSTANCES: usize = 100_000;

#[derive(Debug)]
pub struct Pipeline {
    pipeline: wgpu::RenderPipeline,
    constants: wgpu::BindGroup,
    constants_buffer: wgpu::Buffer,
    vertices: wgpu::Buffer,
    indices: wgpu::Buffer,
    instances: wgpu::Buffer,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Quad {
    pub position: [f32; 2],
    pub size: [f32; 2],
    pub color: [f32; 4],
    pub border_color: [f32; 4],
    pub border_radius: f32,
    pub border_width: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Zeroable, Pod)]
pub struct Vertex {
    _position: [f32; 2],
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
struct Uniforms {
    transform: [f32; 16],
    scale: f32,
    _padding: [f32; 3],
}

impl Pipeline {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Pipeline {
        let constant_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(concat!(module_path!(), " uniforms layout")),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStage::VERTEX,
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
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
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
            flags: wgpu::ShaderFlags::all(),
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(concat!(module_path!(), " pipeline")),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    wgpu::VertexBufferLayout {
                        array_stride: mem::size_of::<Vertex>() as u64,
                        step_mode: wgpu::InputStepMode::Vertex,
                        attributes: &[wgpu::VertexAttribute {
                            shader_location: 0,
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 0,
                        }],
                    },
                    wgpu::VertexBufferLayout {
                        array_stride: mem::size_of::<Quad>() as u64,
                        step_mode: wgpu::InputStepMode::Instance,
                        attributes: &wgpu::vertex_attr_array!(
                            1 => Float32x2,
                            2 => Float32x2,
                            3 => Float32x4,
                            4 => Float32x4,
                            5 => Float32,
                            6 => Float32,
                        ),
                    },
                ],
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
                    write_mask: wgpu::ColorWrite::ALL,
                }],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
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

        let vertices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(concat!(module_path!(), " vertex buffer")),
            contents: bytemuck::cast_slice(&QUAD_VERTS),
            usage: wgpu::BufferUsage::VERTEX,
        });

        let indices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(concat!(module_path!(), " index buffer")),
            contents: bytemuck::cast_slice(&QUAD_INDICES),
            usage: wgpu::BufferUsage::INDEX,
        });

        let instances = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(concat!(module_path!(), " instance buffer")),
            size: mem::size_of::<Quad>() as u64 * MAX_INSTANCES as u64,
            usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST,
            mapped_at_creation: false,
        });

        Pipeline {
            pipeline,
            constants,
            constants_buffer,
            vertices,
            indices,
            instances,
        }
    }

    pub fn draw(
        &mut self,
        device: &wgpu::Device,
        staging_belt: &mut wgpu::util::StagingBelt,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        instances: (&[Quad], &[Quad]),
        bounds: PhysicalRectangle,
        scale_factor: f32,
        transformation: Transformation,
    ) {
        let uniforms = Uniforms::new(transformation, scale_factor);

        {
            let mut constants_buffer = staging_belt.write_buffer(
                encoder,
                &self.constants_buffer,
                0,
                wgpu::BufferSize::new(mem::size_of::<Uniforms>() as u64).unwrap(),
                device,
            );

            constants_buffer.copy_from_slice(bytemuck::bytes_of(&uniforms));
        }

        let mut i = 0;
        let total = instances.0.len() + instances.1.len();

        while i < total {
            let end = (i + MAX_INSTANCES).min(total);
            let amount = end - i;

            let (first_instances, second_instances) =
                select_slices(instances.0, instances.1, i..end);
            let first_instance_bytes = first_instances.map(bytemuck::cast_slice);
            let second_instance_bytes = second_instances.map(bytemuck::cast_slice);
            let total_bytes = first_instance_bytes.map_or(0, |bytes| bytes.len())
                + second_instance_bytes.map_or(0, |bytes| bytes.len());

            let mut instance_buffer = staging_belt.write_buffer(
                encoder,
                &self.instances,
                0,
                wgpu::BufferSize::new(total_bytes as u64).unwrap(),
                device,
            );

            if let Some(bytes) = first_instance_bytes {
                instance_buffer.copy_from_slice(bytes);
            }
            if let Some(bytes) = second_instance_bytes {
                instance_buffer.copy_from_slice(bytes);
            }

            {
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
                render_pass.set_index_buffer(self.indices.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.set_vertex_buffer(0, self.vertices.slice(..));
                render_pass.set_vertex_buffer(1, self.instances.slice(..));

                render_pass.set_scissor_rect(
                    bounds.x as u32,
                    bounds.y as u32,
                    bounds.width,
                    bounds.height,
                );

                render_pass.draw_indexed(0..QUAD_INDICES.len() as u32, 0, 0..amount as u32);
            }

            i += MAX_INSTANCES;
        }
    }
}

unsafe impl bytemuck::Zeroable for Quad {}

unsafe impl bytemuck::Pod for Quad {}

impl Uniforms {
    fn new(transformation: Transformation, scale: f32) -> Uniforms {
        Self {
            transform: *transformation.as_ref(),
            scale,
            _padding: [0.0; 3],
        }
    }
}

impl Default for Uniforms {
    fn default() -> Self {
        Self {
            transform: *Transformation::identity().as_ref(),
            scale: 1.0,
            _padding: [0.0; 3],
        }
    }
}

fn select_slices<'a, 'b, T>(
    first: &'a [T],
    second: &'b [T],
    range: Range<usize>,
) -> (Option<&'a [T]>, Option<&'b [T]>) {
    let first_len = first.len();

    let first_result = if range.start < first_len {
        Some(&first[range.start..range.end.max(first_len - 1)])
    } else {
        None
    };

    let second_result = if range.end > first_len {
        let second_len = second.len();
        Some(&second[(range.start.saturating_sub(second_len))..(range.end - second_len)])
    } else {
        None
    };

    (first_result, second_result)
}
