use crate::util::node::ComputeNode;
use crate::util::node::{ViewNode, ViewNodeBuilder};
use crate::util::{vertex::PosParticleIndex, BufferObj};

use super::{generate_cloth_particles, ClothUniform, MeshColoringObj};

use app_surface::{
    math::{Position, Size},
    AppSurface, SurfaceFrame, Touch, TouchPhase,
};
use zerocopy::AsBytes;

pub struct ClothX {
    particle_buf: BufferObj,
    constraint_buf: BufferObj,
    bend_constraints_buf: BufferObj,

    stretch_mesh_coloring: Vec<MeshColoringObj>,
    bend_mesh_coloring: Vec<MeshColoringObj>,

    // 预测位置并重置约束的 lambda 等参数
    predict_and_reset: ComputeNode,
    stretch_solver: ComputeNode,
    bend_solver: ComputeNode,
    display_node: ViewNode,
    depth_texture_view: wgpu::TextureView,
    frame_count: usize,
    // 迭代次数
    pbd_iter_count: usize,
}

impl ClothX {
    pub fn new(app_view: &AppSurface) -> Self {
        let _encoder =
            app_view.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        let viewport_size: Size<f32> = (&app_view.config).into();
        let (proj_mat, mv_mat, factor) =
            crate::util::utils::matrix_helper::perspective_mvp(viewport_size);
        let mvp_buf = BufferObj::create_uniform_buffer(
            &app_view.device,
            &crate::MVPMatUniform {
                mv: mv_mat.into(),
                proj: proj_mat.into(),
                mvp: (proj_mat * mv_mat).into(),
                normal: mv_mat.into(),
            },
            None,
        );
        // （32， 64） 这个组合，约束分组后为 9 组，且没有极小数据量的分组
        // 为何 （32， 64）导致在 particle buffer 里开始部分数据顺序乱了？
        let particle_x_num = 32_u32;
        let particle_y_num = 58_u32;

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
        let pbd_iter_count = 15;
        let delta_time = 0.016 / pbd_iter_count as f32;
        let uniform_buf = BufferObj::create_uniform_buffer(
            &app_view.device,
            &ClothUniform {
                num_x: particle_x_num as i32,
                num_y: particle_y_num as i32,
                triangle_num: 0,
                compliance: 0.0000000016 / (delta_time * delta_time),
                dt: delta_time,
            },
            Some("cloth uniform"),
        );
        let (
            (_tl_x, _tl_y),
            particles,
            constraints,
            stretch_mesh_coloring,
            _particle_constraints,
            reorder_constraints,
            bend_mesh_coloring,
            bend_constraints,
            reorder_bendings,
        ) = generate_cloth_particles(
            particle_x_num as usize,
            particle_y_num as usize,
            app_view.config.width as f32,
            app_view.config.width as f32 / 1863.0 * 3312.0,
            factor.0 / viewport_size.width,
        );
        // dynamit uniform
        let dynamic_offset =
            app_view.device.limits().min_uniform_buffer_offset_alignment as wgpu::BufferAddress;
        let stretch_coloring_buf = BufferObj::create_empty_dynamic_uniform_buffer(
            &app_view.device,
            stretch_mesh_coloring.len() as u64 * dynamic_offset,
            None,
            Some("stretch_coloring_buf"),
        );
        let mut offset = 0;
        for mc in stretch_mesh_coloring.iter() {
            app_view.queue.write_buffer(
                &stretch_coloring_buf.buffer,
                offset,
                mc.get_push_constants_data().as_bytes(),
            );
            offset += dynamic_offset;
        }

        let bend_coloring_buf = BufferObj::create_empty_dynamic_uniform_buffer(
            &app_view.device,
            bend_mesh_coloring.len() as u64 * dynamic_offset * pbd_iter_count as u64,
            None,
            Some("bend_coloring_buf"),
        );
        offset = 0;
        for i in 0..pbd_iter_count {
            for mc in bend_mesh_coloring.iter() {
                app_view.queue.write_buffer(
                    &bend_coloring_buf.buffer,
                    offset,
                    mc.get_bending_dynamic_uniform(i).as_bytes(),
                );
                offset += dynamic_offset;
            }
        }

        let predict_dynamic_buf = BufferObj::create_empty_dynamic_uniform_buffer(
            &app_view.device,
            2 * dynamic_offset,
            None,
            Some("predict_dynamic_buf"),
        );
        for i in 0..2 {
            app_view.queue.write_buffer(
                &predict_dynamic_buf.buffer,
                i * dynamic_offset,
                vec![i].as_bytes(),
            );
            offset += dynamic_offset;
        }
        let particle_buf =
            BufferObj::create_storage_buffer(&app_view.device, &particles, Some("particle buf"));

        let constraint_buf = BufferObj::create_storage_buffer(
            &app_view.device,
            &constraints,
            Some("constraint_buf"),
        );

        let reorder_constraints_buf = BufferObj::create_storage_buffer(
            &app_view.device,
            &reorder_constraints,
            Some("reorder_constraints_buf"),
        );
        let bend_constraints_buf = BufferObj::create_storage_buffer(
            &app_view.device,
            &bend_constraints,
            Some("bend_constraints_buf"),
        );
        let reorder_bendings_buf = BufferObj::create_storage_buffer(
            &app_view.device,
            &reorder_bendings,
            Some("reorder_bendings_buf"),
        );
        let predict_and_reset_shader = crate::util::shader::create_shader_module(
            &app_view.device,
            "pbd/xxpbd/cloth_predict",
            None,
        );
        let predict_and_reset = ComputeNode::new_with_dynamic_uniforms(
            &app_view.device,
            (((particle_x_num * particle_y_num + 31) as f32 / 32.0).floor() as u32, 1, 1),
            vec![&uniform_buf],
            vec![&predict_dynamic_buf],
            vec![&particle_buf, &constraint_buf, &reorder_constraints_buf],
            vec![],
            &predict_and_reset_shader,
        );

        let constraint_solver_shader = crate::util::shader::create_shader_module(
            &app_view.device,
            "pbd/xxpbd/cloth_stretch_solver",
            None,
        );
        let stretch_solver = ComputeNode::new_with_dynamic_uniforms(
            &app_view.device,
            (0, 0, 0),
            vec![&uniform_buf],
            vec![(&stretch_coloring_buf)],
            vec![&particle_buf, &constraint_buf, &reorder_constraints_buf],
            vec![],
            &constraint_solver_shader,
        );

        let bend_solver_shader = crate::util::shader::create_shader_module(
            &app_view.device,
            "pbd/xxpbd/cloth_bending_solver",
            None,
        );
        let size = particle_x_num * particle_y_num * 4 * 16;
        let _debug_buf = BufferObj::create_empty_storage_buffer(
            &app_view.device,
            size as wgpu::BufferAddress,
            false,
            Some("debug_buf"),
        );
        let bend_solver = ComputeNode::new_with_dynamic_uniforms(
            &app_view.device,
            (0, 0, 0),
            vec![&uniform_buf],
            vec![&bend_coloring_buf],
            vec![&particle_buf, &bend_constraints_buf, &reorder_bendings_buf],
            vec![],
            &bend_solver_shader,
        );

        let mut vertex_data: Vec<PosParticleIndex> = Vec::new();
        let mut index_data: Vec<u32> = Vec::new();
        // 按行遍历
        for h in 0..particle_y_num {
            for w in 0..particle_x_num {
                vertex_data.push(PosParticleIndex::new([w, h, 0]));
                if h > 0 && w > 0 {
                    let current: u32 = particle_x_num * h + w;
                    // 找到上一行同一行位置的索引
                    let top: u32 = current - particle_x_num;
                    let mut lines: Vec<u32> =
                        vec![current, top, top - 1, current, top - 1, current - 1];
                    index_data.append(&mut lines);
                }
            }
        }
        // 1863*3312
        // let img_path = PathBuf::from(&base_path).join("assets/paper/3.png");

        let (texture, _) = crate::util::load_texture::from_path(
            "dragon.png",
            app_view,
            wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
            false,
        );
        let display_shader =
            crate::util::shader::create_shader_module(&app_view.device, "pbd/cloth_display", None);
        let display_node_builder =
            ViewNodeBuilder::<PosParticleIndex>::new(vec![(&texture, None)], &display_shader)
                .with_uniform_buffers(vec![&mvp_buf, &uniform_buf])
                .with_storage_buffers(vec![&particle_buf])
                .with_use_depth_stencil(true)
                .with_cull_mode(None)
                .with_shader_stages(vec![
                    wgpu::ShaderStages::VERTEX,
                    wgpu::ShaderStages::VERTEX,
                    wgpu::ShaderStages::VERTEX,
                    wgpu::ShaderStages::FRAGMENT,
                    wgpu::ShaderStages::FRAGMENT,
                ])
                .with_vertices_and_indices((vertex_data, index_data));

        let display_node = display_node_builder.build(&app_view.device);

        let size = wgpu::Extent3d {
            width: app_view.config.width,
            height: app_view.config.height,
            depth_or_array_layers: 1,
        };
        let depth_texture_view =
            crate::util::depth_stencil::create_depth_texture_view(size, &app_view.device);

        let instance = Self {
            particle_buf,
            constraint_buf,
            stretch_mesh_coloring,
            bend_mesh_coloring,
            bend_constraints_buf,
            predict_and_reset,
            stretch_solver,
            bend_solver,
            display_node,
            depth_texture_view,
            frame_count: 0,
            pbd_iter_count: pbd_iter_count as usize,
        };

        instance
    }

    fn step_solver(&mut self, encoder: &mut wgpu::CommandEncoder) {
        // if self.frame_count >= 1 {
        //     return;
        // }
        // 重用 cpass 在 macOS 上不能提升性能， 但是在 iOS 上提升明显
        // 64*64，8 约束，迭代20 ：Xs Max, 12ms -> 8ms
        let mut cpass =
            encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("solver pass") });

        let dynamic_offset = 256;
        for i in 0..self.pbd_iter_count {
            // 下一次迭代的开始，先更新粒子速度
            let offset = if i == 0 { 256 } else { 0 };
            self.predict_and_reset.dispatch_by_offsets(&mut cpass, Some(vec![vec![offset]]));

            cpass.set_pipeline(&self.stretch_solver.pipeline);
            cpass.set_bind_group(0, &self.stretch_solver.bg_setting.bind_group, &[]);
            let mut index = 0;
            for mc in self.stretch_mesh_coloring.iter() {
                if let Some(bg) = &self.stretch_solver.dy_uniform_bg {
                    cpass.set_bind_group(1, &bg.bind_group, &[index * dynamic_offset]);
                }
                cpass.dispatch_workgroups(mc.thread_group.0, mc.thread_group.1, 1);
                index += 1;
            }

            let bending_dynamic_uniform_offset =
                (i * self.bend_mesh_coloring.len() * dynamic_offset as usize)
                    as wgpu::DynamicOffset;
            cpass.set_pipeline(&self.bend_solver.pipeline);
            cpass.set_bind_group(0, &self.bend_solver.bg_setting.bind_group, &[]);
            index = 0;
            for mc in self.bend_mesh_coloring.iter() {
                if let Some(bg) = &self.bend_solver.dy_uniform_bg {
                    cpass.set_bind_group(
                        1,
                        &bg.bind_group,
                        &[bending_dynamic_uniform_offset + index * dynamic_offset],
                    );
                }
                cpass.dispatch_workgroups(mc.thread_group.0, mc.thread_group.1, 1);
                index += 1;
            }
        }

        self.frame_count += 1;
    }

    pub fn enter_frame(&mut self, app_view: &mut AppSurface) {
        let mut encoder = app_view.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("cloth encoder"),
        });
        self.step_solver(&mut encoder);
        let (frame, frame_view) = app_view.get_current_frame_view();
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("cloth render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &frame_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(crate::util::utils::alpha_color()),
                        store: true,
                    },
                })],
                // depth_stencil_attachment: None,
                depth_stencil_attachment: Some(
                    crate::util::utils::depth_stencil::create_attachment(&self.depth_texture_view),
                ),
            });
            self.display_node.draw_render_pass(&mut rpass);
        }
        app_view.queue.submit(Some(encoder.finish()));
        frame.present();
    }
    pub fn rotate(&mut self, _app_view: &app_surface::AppSurface, _x: f32, _y: f32) {}
}
