use idroid::node::{BufferlessFullscreenNode, ComputeNode, ViewNode, ViewNodeBuilder};
use idroid::{math::Size, vertex::VertexEmpty, BufferObj};

use super::{
    bristle::{generate_bristles, BristleParticle},
    MeshColoringObj,
};
use nalgebra_glm as glm;
use uni_view::{AppView, GPUContext};

use zerocopy::AsBytes;

pub struct MaoBrush {
    mvp_buf: BufferObj,
    translate_z: f32,
    proj_mat: glm::TMat4<f32>,
    particle_buf: BufferObj,
    predict_solver: ComputeNode,

    stretch_constraints_buf: BufferObj,
    stretch_constraints_group_buf: BufferObj,
    stretch_mesh_coloring: Vec<MeshColoringObj>,
    stretch_solver: ComputeNode,

    bend_constraints_buf: BufferObj,
    bend_mesh_coloring: Vec<MeshColoringObj>,
    bend_solver: ComputeNode,

    debug_plane: BufferlessFullscreenNode,

    display_node: ViewNode,
    depth_texture_view: wgpu::TextureView,
    frame_count: usize,
    // 迭代次数
    pbd_iter_count: usize,
}

impl MaoBrush {
    pub fn new(app_view: &AppView) -> Self {
        let viewport_size: Size<f32> = (&app_view.config).into();
        let (proj_mat, _mv_mat, factor) =
            idroid::utils::matrix_helper::perspective_mvp(viewport_size);
        // change mv_mat's z to 0
        let translate_z = factor.2 - 0.6;
        let mut model_rotate_mat = glm::rotate_x(&glm::Mat4::identity(), -0.8);
        model_rotate_mat = glm::rotate_y(&model_rotate_mat, -0.8);
        let translate_mat =
            glm::translate(&glm::TMat4::<f32>::identity(), &glm::vec3(0.0, 0.0, translate_z));
        // let new_mv_mat = translate_mat * model_rotate_mat;
        let new_mv_mat = translate_mat;

        let mvp_buf = BufferObj::create_uniform_buffer(
            &app_view.device,
            &crate::MVPMatUniform {
                mv: new_mv_mat.into(),
                proj: proj_mat.into(),
                mvp: (proj_mat * new_mv_mat).into(),
                normal: new_mv_mat.into(),
            },
            None,
        );

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

        // 由于一个粒子只有 上， 右 两个 stretch 约束，迭代次数小于 20 会出现严重的位置错误
        let pbd_iter_count = 20;
        let delta_time = 0.016 / pbd_iter_count as f32;
        let uniform_buf = BufferObj::create_uniform_buffer(
            &app_view.device,
            &[0.0000000016 / (delta_time * delta_time), delta_time],
            Some("brush uniform"),
        );
        let (
            particles,
            index_data,
            stretches,
            reorder_streches,
            stretch_mesh_coloring,
            bending_constraints,
            reorder_bendings,
            bend_mesh_coloring,
        ) = generate_bristles(0.0);
        let particles = particles
            .iter()
            .map(|&x| {
                let invert_mass = x.pos[3];
                let mut pos: [f32; 4] =
                    (model_rotate_mat * glm::vec4(x.pos[0], x.pos[1], x.pos[2], 1.0)).into();
                pos[3] = invert_mass;
                BristleParticle { pos: pos, old_pos: pos, connect: x.connect }
            })
            .collect::<Vec<_>>();
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

        let particle_buf = BufferObj::create_buffer(
            &app_view.device,
            Some(&particles),
            None,
            wgpu::BufferUsages::STORAGE,
            Some("particle_buffer"),
        );

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
        let predict_shader =
            idroid::shader::create_shader_module(&app_view.device, "pbd/brush/predict", None);
        let predict_solver = ComputeNode::new_with_dynamic_uniforms(
            &app_view.device,
            (((20 * 13 + 31) as f32 / 32.0).floor() as u32, 1, 1),
            vec![&uniform_buf],
            vec![&predict_dynamic_buf],
            vec![&particle_buf],
            vec![],
            &predict_shader,
        );

        let stretch_constraints_buf = BufferObj::create_storage_buffer(
            &app_view.device,
            &stretches,
            Some("stretch_constraints_buf"),
        );
        let stretch_constraints_group_buf = BufferObj::create_storage_buffer(
            &app_view.device,
            &reorder_streches,
            Some("stretch_constraints_group_buf"),
        );
        let stretch_solver_shader = idroid::shader::create_shader_module(
            &app_view.device,
            "pbd/brush/stretch_solver",
            None,
        );
        let stretch_solver = ComputeNode::new_with_dynamic_uniforms(
            &app_view.device,
            (0, 0, 0),
            vec![&uniform_buf],
            vec![(&stretch_coloring_buf)],
            vec![&particle_buf, &stretch_constraints_buf, &stretch_constraints_group_buf],
            vec![],
            &stretch_solver_shader,
        );

        let bend_constraints_buf = BufferObj::create_storage_buffer(
            &app_view.device,
            &bending_constraints,
            Some("bend_constraints_buf"),
        );
        let reorder_bendings_buf = BufferObj::create_storage_buffer(
            &app_view.device,
            &reorder_bendings,
            Some("reorder_bendings_buf"),
        );
        let bend_solver_shader = idroid::shader::create_shader_module(
            &app_view.device,
            "pbd/brush/bending_solver",
            None,
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

        let bufferless_shader = idroid::shader::create_shader_module(
            &app_view.device,
            "pbd/brush/debug_plane",
            Some("bufferless"),
        );
        let debug_plane = BufferlessFullscreenNode::new(
            &app_view.device,
            app_view.config.format,
            vec![&mvp_buf],
            vec![],
            vec![],
            vec![],
            &bufferless_shader,
            None,
            true,
        );

        let display_shader =
            idroid::shader::create_shader_module(&app_view.device, "pbd/brush/display", None);
        let display_node_builder = ViewNodeBuilder::<VertexEmpty>::new(vec![], &display_shader)
            .with_uniform_buffers(vec![&mvp_buf, &uniform_buf])
            .with_storage_buffers(vec![&particle_buf])
            .with_vertices_and_indices((vec![VertexEmpty {}], index_data))
            .with_use_depth_stencil(true)
            .with_cull_mode(None)
            .with_shader_stages(vec![
                wgpu::ShaderStages::VERTEX,
                wgpu::ShaderStages::VERTEX,
                wgpu::ShaderStages::VERTEX,
            ])
            .with_primitive_topology(wgpu::PrimitiveTopology::LineStrip);

        let display_node = display_node_builder.build(&app_view.device);

        let size = wgpu::Extent3d {
            width: app_view.config.width,
            height: app_view.config.height,
            depth_or_array_layers: 1,
        };
        let depth_texture_view =
            idroid::depth_stencil::create_depth_texture_view(size, &app_view.device);

        let instance = Self {
            mvp_buf,
            translate_z,
            proj_mat,
            particle_buf,
            predict_solver,
            stretch_constraints_buf,
            stretch_constraints_group_buf,
            stretch_mesh_coloring,
            stretch_solver,
            bend_constraints_buf,
            bend_mesh_coloring,
            bend_solver,
            display_node,
            debug_plane,
            depth_texture_view,
            frame_count: 0,
            pbd_iter_count: pbd_iter_count as usize,
        };

        instance
    }

    pub fn rotate(&mut self, app_view: &idroid::AppView, x: f32, y: f32) {
        let mut model_rotate_mat = glm::rotate_x(&glm::Mat4::identity(), 0.8 * x);
        model_rotate_mat = glm::rotate_y(&model_rotate_mat, 0.8 * y);

        let translate_mat =
            glm::translate(&glm::TMat4::<f32>::identity(), &glm::vec3(0.0, 0.0, self.translate_z));
        let new_mv_mat = translate_mat * model_rotate_mat;

        let normal: [[f32; 4]; 4] = glm::inverse_transpose(new_mv_mat).into();
        let mvp_uniform = crate::MVPMatUniform {
            mv: new_mv_mat.into(),
            proj: self.proj_mat.into(),
            mvp: (self.proj_mat * new_mv_mat).into(),
            normal: normal,
        };
        app_view.queue.write_buffer(&self.mvp_buf.buffer, 0, &mvp_uniform.as_bytes());
    }

    fn step_solver(&mut self, encoder: &mut wgpu::CommandEncoder) {
        // if self.frame_count >= 1 {
        //     return;
        // }

        let mut cpass =
            encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("solver pass") });

        let dynamic_offset = 256;
        for i in 0..self.pbd_iter_count {
            // 下一次迭代的开始，先更新粒子偏移
            let offset = if i == 0 { 256 } else { 0 };
            self.predict_solver.dispatch_by_offsets(&mut cpass, Some(vec![vec![offset]]));

            cpass.set_pipeline(&self.stretch_solver.pipeline);
            cpass.set_bind_group(0, &self.stretch_solver.bg_setting.bind_group, &[]);
            let mut index = 0;
            for mc in self.stretch_mesh_coloring.iter() {
                if let Some(bg) = &self.stretch_solver.dy_uniform_bg {
                    cpass.set_bind_group(1, &bg.bind_group, &[index * dynamic_offset]);
                }
                cpass.dispatch(mc.thread_group.0, mc.thread_group.1, 1);
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
                cpass.dispatch(mc.thread_group.0, mc.thread_group.1, 1);
                index += 1;
            }
        }

        self.frame_count += 1;
    }

    pub fn enter_frame(&mut self, app_view: &mut AppView) {
        let mut encoder = app_view.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("MaoBrush encoder"),
        });
        self.step_solver(&mut encoder);

        let (frame, frame_view) = app_view.get_current_frame_view();
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("MaoBrush render pass"),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &frame_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(idroid::utils::alpha_color()),
                        store: true,
                    },
                }],
                // depth_stencil_attachment: None,
                depth_stencil_attachment: Some(idroid::utils::depth_stencil::create_attachment(
                    &self.depth_texture_view,
                )),
            });
            self.display_node.draw_render_pass(&mut rpass);

            let w = app_view.config.width as f32;
            let h = app_view.config.height as f32;
            let x = (w - 200.0) / 2.0;
            let y = (h - 200.0) / 2.0;
            rpass.set_viewport(x, y, 200.0, 200.0, 0.0, 0.0);
            self.debug_plane.draw_rpass(&mut rpass);
        }
        app_view.queue.submit(Some(encoder.finish()));
        frame.present();
    }
}
