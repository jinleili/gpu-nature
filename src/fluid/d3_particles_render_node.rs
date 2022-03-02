use crate::util::BufferObj;
use app_surface::math::Size;
use wgpu::util::DeviceExt;
use zerocopy::AsBytes;

use crate::{create_shader_module, TrajectoryUniform};

pub struct D3ParticleRenderNode {
    update_bind_group: wgpu::BindGroup,
    update_pipeline: wgpu::RenderPipeline,

    bind_group: wgpu::BindGroup,
    compose_pipeline: wgpu::RenderPipeline,
    vertices_buf: wgpu::Buffer,
    points_buf: wgpu::Buffer,

    particles_buf: BufferObj,
    frame_index: u32,
}

impl D3ParticleRenderNode {
    pub fn new(
        device: &wgpu::Device, canvas_format: wgpu::TextureFormat, point_size: f32,
        canvas_size: Size<u32>, macro_tex: &crate::util::AnyTexture,
        depth_tex: &crate::util::AnyTexture,
    ) -> Self {
        let sampler = crate::util::load_texture::bilinear_sampler(device);

        let uniform_data = TrajectoryUniform {
            screen_factor: [2.0 / canvas_size.width as f32, 2.0 / canvas_size.height as f32],
            trajectory_view_index: 0,
            bg_view_index: 1,
        };
        let uniform_buf = BufferObj::create_uniform_buffer(
            device,
            &uniform_data,
            Some("particle render uniform_buf"),
        );
        let layouts: Vec<wgpu::BindGroupLayoutEntry> = vec![
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(0),
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D3,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ];
        let entries: Vec<wgpu::BindGroupEntry> = vec![
            wgpu::BindGroupEntry { binding: 0, resource: uniform_buf.buffer.as_entire_binding() },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&macro_tex.tex_view),
            },
            wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Sampler(&sampler) },
        ];

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &layouts,
            label: None,
        });
        let bind_group: wgpu::BindGroup = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &entries,
            label: None,
        });

        let half_size = point_size / 2.0;
        let vertex_buffer_data = [
            half_size, half_size, half_size, -half_size, -half_size, half_size, -half_size,
            -half_size,
        ];
        let vertices_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: vertex_buffer_data.as_bytes(),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let points_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("points Buffer"),
            contents: [0.0, 0.0].as_bytes(),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let compose_shader = create_shader_module(
            device,
            "3d_lbm/particles_present",
            Some("particles_present shader"),
        );
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("render"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

        let compose_pipeline = generate_pipeline(
            device,
            vec![canvas_format.into()],
            &render_pipeline_layout,
            &compose_shader,
            ("vs_compose", "fs_compose"),
            false,
            Some(depth_tex),
        );

        let particles_data = crate::init_3d_particles(wgpu::Extent3d {
            width: 100,
            height: 100,
            depth_or_array_layers: 20,
        });
        let particles_buf = BufferObj::create_buffer(
            device,
            Some(&particles_data.as_bytes()),
            None,
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::VERTEX,
            Some("particles_buf"),
        );

        // particles update pipeline
        let mut update_layouts = layouts.clone();
        update_layouts.push(wgpu::BindGroupLayoutEntry {
            binding: 3,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: wgpu::BufferSize::new(0),
            },
            count: None,
        });
        let mut update_entries = entries.clone();
        update_entries.push(wgpu::BindGroupEntry {
            binding: 3,
            resource: particles_buf.buffer.as_entire_binding(),
        });

        let update_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &update_layouts,
            label: None,
        });
        let update_bind_group: wgpu::BindGroup =
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &update_bgl,
                entries: &update_entries,
                label: None,
            });

        let update_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("update"),
                bind_group_layouts: &[&update_bgl],
                push_constant_ranges: &[],
            });
        let update_shader = create_shader_module(
            device,
            "3d_lbm/particles_update",
            Some("particles_update shader"),
        );
        let update_pipeline = generate_pipeline(
            device,
            vec![canvas_format.into()],
            &update_pipeline_layout,
            &update_shader,
            ("vs_update", "fs_update"),
            true,
            Some(depth_tex),
        );

        D3ParticleRenderNode {
            update_bind_group,
            update_pipeline,
            bind_group,
            compose_pipeline,
            vertices_buf,
            points_buf,
            particles_buf,
            frame_index: 0,
        }
    }

    pub fn update_particles(
        &self, encoder: &mut wgpu::CommandEncoder, frame_view: &wgpu::TextureView,
    ) {
        // update
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: frame_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.2, g: 0.2, b: 0.25, a: 1.0 }),
                    store: false,
                },
            }],
            depth_stencil_attachment: None,
        });
        rpass.set_pipeline(&self.update_pipeline);
        rpass.set_bind_group(0, &self.update_bind_group, &[]);
        // rpass.set_vertex_buffer(0, self.points_buf.slice(..));
        rpass.draw(0..1, 0..200000);
    }

    pub fn draw_rpass<'a, 'b: 'a>(&'b self, rpass: &mut wgpu::RenderPass<'a>) {
        // compose
        rpass.set_pipeline(&self.compose_pipeline);
        rpass.set_bind_group(0, &self.bind_group, &[]);
        rpass.set_vertex_buffer(0, self.particles_buf.buffer.slice(..));
        rpass.set_vertex_buffer(1, self.vertices_buf.slice(..));
        rpass.draw(0..4, 0..200000);
    }
}

fn generate_pipeline(
    device: &wgpu::Device, targets: Vec<wgpu::ColorTargetState>,
    pipeline_layout: &wgpu::PipelineLayout, shader: &wgpu::ShaderModule,
    entry_points: (&'static str, &'static str), is_update: bool,
    depth_tex: Option<&crate::util::AnyTexture>,
) -> wgpu::RenderPipeline {
    let attributes = wgpu::vertex_attr_array![0 => Float32x4, 1 => Float32x4];
    let buffers: Vec<wgpu::VertexBufferLayout> = if is_update {
        vec![]
    } else {
        vec![
            wgpu::VertexBufferLayout {
                array_stride: 4 * 8,
                step_mode: wgpu::VertexStepMode::Instance,
                attributes: &attributes,
            },
            wgpu::VertexBufferLayout {
                array_stride: 2 * 4,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &wgpu::vertex_attr_array![2 => Float32x2],
            },
        ]
    };
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(pipeline_layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: entry_points.0,
            buffers: &buffers,
        },
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: entry_points.1,
            targets: &targets,
        }),
        primitive: wgpu::PrimitiveState {
            topology: if is_update {
                wgpu::PrimitiveTopology::PointList
            } else {
                wgpu::PrimitiveTopology::TriangleStrip
            },
            front_face: wgpu::FrontFace::Cw,
            cull_mode: None,
            polygon_mode: if is_update {
                wgpu::PolygonMode::Point
            } else {
                wgpu::PolygonMode::Fill
            },
            ..Default::default()
        },
        depth_stencil: if is_update {
            None
        } else {
            if let Some(depth_tex) = depth_tex {
                Some(wgpu::DepthStencilState {
                    format: depth_tex.format,
                    depth_write_enabled: false,
                    depth_compare: wgpu::CompareFunction::LessEqual,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                })
            } else {
                None
            }
        },
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
    })
}
