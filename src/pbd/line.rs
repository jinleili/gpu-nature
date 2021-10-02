use idroid::node::ComputeNode;
use idroid::node::{ViewNode, ViewNodeBuilder};
use idroid::{math::Size, vertex::PosParticleIndex, BufferObj, MVPUniform, MVPUniformObj};

use super::{generate_line_particles, ClothUniform};
use std::path::PathBuf;
use uni_view::{fs::FileSystem, AppView, GPUContext};
use zerocopy::{AsBytes, FromBytes};

pub struct Line {
    particle_buf: BufferObj,
    constraint_buf: BufferObj,
    // 预测位置并重置约束的 lambda 等参数
    predict_and_reset: ComputeNode,
    constraint_solver: ComputeNode,
    display_node: ViewNode,
    group_count: (u32, u32, u32),
}

impl Line {
    pub fn new(app_view: &AppView) -> Self {
        let mut encoder = app_view.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        let viewport_size: Size<f32> = (&app_view.config).into();
        let (p_matrix, base_mv_matrix, factor) = idroid::utils::matrix_helper::perspective_mvp(viewport_size);
        let mvp_buf = BufferObj::create_uniform_buffer(
            &app_view.device,
            &MVPUniform { mvp_matrix: (p_matrix * base_mv_matrix).into() },
            None,
        );
        let line_num = 64_u32;
        let line_particle_num = 16_u32;
        let group_count = (line_num / 8, 2, 1);

        //static const float MODE_COMPLIANCE[eModeMax] = {
        //  0.0f,            // Miles Macklin's blog (http://blog.mmacklin.com/2016/10/12/xpbd-slides-and-stiffness/)
        //  0.00000000004f, // 0.04 x 10^(-9) (M^2/N) Concrete
        //  0.00000000016f, // 0.16 x 10^(-9) (M^2/N) Wood
        //  0.000000001f,   // 1.0  x 10^(-8) (M^2/N) Leather
        //  0.000000002f,   // 0.2  x 10^(-7) (M^2/N) Tendon
        //  0.0000001f,     // 1.0  x 10^(-6) (M^2/N) Rubber
        //  0.00002f,       // 0.2  x 10^(-3) (M^2/N) Muscle
        //  0.0001f,        // 1.0  x 10^(-3) (M^2/N) Fat
        //};
        let uniform_buf = BufferObj::create_uniform_buffer(
            &app_view.device,
            &ClothUniform {
                num_x: line_num as i32,
                num_y: line_particle_num as i32,
                triangle_num: 0,
                compliance: 0.0000001,
                dt: 0.016,
            },
            None,
        );

        let (particles, constraints) = generate_line_particles(
            line_num as usize,
            line_particle_num as usize,
            660.0,
            1100.0,
            factor.0 / viewport_size.width,
        );

        let particle_buf =
            BufferObj::create_storage_buffer(&app_view.device, particles.as_bytes(), Some("particle_buf"));
        let constraint_buf =
            BufferObj::create_storage_buffer(&app_view.device, constraints.as_bytes(), Some("constraint_buf"));

        let predict_and_reset_shader =
            idroid::shader::create_shader_module(&app_view.device, "pbd/line_predict_and_reset", None);
        let predict_and_reset = ComputeNode::new(
            &app_view.device,
            group_count,
            vec![&uniform_buf],
            vec![&particle_buf, &constraint_buf],
            vec![],
            &predict_and_reset_shader,
        );

        let constraint_solver_shader =
            idroid::shader::create_shader_module(&app_view.device, "pbd/line_constraint_solver", None);
        let constraint_solver = ComputeNode::new_with_push_constants(
            &app_view.device,
            (group_count.0, 1, 1),
            vec![&uniform_buf],
            vec![&particle_buf, &constraint_buf],
            vec![],
            &constraint_solver_shader,
            Some(vec![(wgpu::ShaderStages::COMPUTE, 0..4)]),
        );

        let mut vertex_data: Vec<PosParticleIndex> = Vec::new();
        let mut index_data: Vec<u32> = Vec::new();
        // 按线条遍历
        for line in 0..line_num {
            for n in 0..line_particle_num {
                vertex_data.push(PosParticleIndex::new([n, line, 0]));
                let current: u32 = line * line_particle_num + n;
                if n < line_particle_num - 1 {
                    index_data.push(current);
                } else {
                    // 图元重启
                    index_data.push((2_u64.pow(32) - 1) as u32);
                }
            }
        }

        let display_shader = idroid::shader::create_shader_module(&app_view.device, "pbd/line_display", None);
        let display_node_builder = ViewNodeBuilder::<PosParticleIndex>::new(vec![], &display_shader)
            .with_uniform_buffers(vec![&mvp_buf, &uniform_buf])
            .with_storage_buffers(vec![&particle_buf])
            .with_shader_stages(vec![
                wgpu::ShaderStages::VERTEX,
                wgpu::ShaderStages::VERTEX,
                wgpu::ShaderStages::VERTEX,
            ])
            .with_vertices_and_indices((vertex_data, index_data))
            .with_primitive_topology(wgpu::PrimitiveTopology::LineStrip);

        let display_node = display_node_builder.build(&app_view.device);
        app_view.queue.submit(Some(encoder.finish()));

        Self { particle_buf, constraint_buf, predict_and_reset, constraint_solver, display_node, group_count }
    }
    pub fn enter_frame(&mut self, app_view: &mut AppView) {
        let mut encoder =
            app_view.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("cloth encoder") });
        self.predict_and_reset.compute(&mut encoder);
        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
            cpass.set_pipeline(&self.constraint_solver.pipeline);
            cpass.set_bind_group(0, &self.constraint_solver.bg_setting.bind_group, &[]);
            for _ in 0..20 {
                cpass.set_push_constants(0, vec![0].as_bytes());
                cpass.dispatch(
                    self.constraint_solver.group_count.0,
                    self.constraint_solver.group_count.1,
                    1,
                );
                cpass.set_push_constants(0, vec![1].as_bytes());
                cpass.dispatch(
                    self.constraint_solver.group_count.0,
                    self.constraint_solver.group_count.1,
                    1,
                );
            }
        }

        let (_frame, frame_view) = app_view.get_current_frame_view();

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &frame_view,
                    resolve_target: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Clear(idroid::utils::alpha_color()), store: true },
                }],
                depth_stencil_attachment: None,
            });
            self.display_node.draw_render_pass(&mut rpass);
        }
        app_view.queue.submit(Some(encoder.finish()));
    }
}
