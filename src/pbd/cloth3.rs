use idroid::node::ComputeNode;
use idroid::node::{ViewNode, ViewNodeBuilder};
use idroid::{math::Size, vertex::PosParticleIndex, BufferObj, MVPUniform2};

use super::{
    generate_cloth_particles3, BinUniform, ClothUniform, FrameUniform, MeshColoringObj, TriangleObj,
};
use nalgebra_glm as glm;
use std::{path::PathBuf, u32};
use uni_view::{fs::FileSystem, AppView, GPUContext};
use zerocopy::AsBytes;

pub struct Cloth3 {
    frame_uniform_buf: BufferObj,
    mesh_coloring: Vec<MeshColoringObj>,
    // 预测位置并重置约束的 lambda 等参数
    predict_and_reset: ComputeNode,
    // 把顶点装入空间hash格子
    binning_node: ComputeNode,
    // 生成碰撞约束
    gen_collision: ComputeNode,
    constraints_solver: ComputeNode,
    display_node: ViewNode,
    point_node: ViewNode,
    normal_line_node: ViewNode,

    depth_texture_view: wgpu::TextureView,
    frame_count: usize,
}

impl Cloth3 {
    pub fn new(app_view: &AppView) -> Self {
        let mut encoder =
            app_view.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        let viewport_size: Size<f32> = (&app_view.config).into();
        let (p_matrix, base_mv_matrix, factor) =
            idroid::utils::matrix_helper::perspective_mvp(viewport_size);
        let mut mv_matrix = glm::rotate_y(&base_mv_matrix, 1.35);
        mv_matrix = glm::rotate_x(&mv_matrix, 0.25);

        let frame_uniform_buf = BufferObj::create_uniform_buffer(
            &app_view.device,
            &FrameUniform { frame_index: 0 },
            None,
        );

        let mvp_buf = BufferObj::create_uniform_buffer(
            &app_view.device,
            &MVPUniform2 { p_matrix: p_matrix.into(), mv_matrix: mv_matrix.into() },
            None,
        );

        let pixel_on_ndc = if viewport_size.width < viewport_size.height {
            2.0 / viewport_size.width
        } else {
            2.0 / viewport_size.height
        };
        // （32， 64） 这个组合，约束分组后为 9 组，且没有极小数据量的分组
        let particle_x_num = 32_u32;
        let particle_y_num = (particle_x_num as f32 * (3312.0 / 1863.0)) as u32;
        let particle_num = particle_x_num * particle_y_num;

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

        let cloth_width = viewport_size.width * 0.7;
        let (
            (tl_x, tl_y, average_edge),
            particles,
            particle_constraints,
            stretch_constraints,
            bending_constraints,
            mesh_coloring,
            mesh_coloring_buf_data,
        ) = generate_cloth_particles3(
            particle_x_num as usize,
            particle_y_num as usize,
            cloth_width,
            cloth_width / 1863.0 * 3312.0,
            pixel_on_ndc,
        );
        let mut vertex_data: Vec<PosParticleIndex> = Vec::new();
        // 法线数据
        let mut normal_data: Vec<PosParticleIndex> = Vec::new();
        let mut index_data: Vec<u32> = Vec::new();
        let mut point_index_data: Vec<u32> = Vec::new();
        let mut normal_index_data: Vec<u32> = Vec::new();

        let mut triangles: Vec<TriangleObj> = vec![];
        // 顶点索引及三角形
        for h in 0..particle_y_num {
            for w in 0..particle_x_num {
                vertex_data.push(PosParticleIndex::new([w, h, 0]));
                normal_data.push(PosParticleIndex::new([w, h, 0]));
                normal_data.push(PosParticleIndex::new([w, h, 1]));
                let index: u32 = particle_x_num * h + w;
                normal_index_data.push(index * 2);
                normal_index_data.push(index * 2 + 1);

                point_index_data.push(index);
                if h > 0 && w > 0 {
                    let current: u32 = index;
                    // 找到上一行同一行位置的索引
                    let top: u32 = current - particle_x_num;
                    let mut lines: Vec<u32> =
                        vec![current, top, top - 1, current, top - 1, current - 1];
                    for i in 0..2 {
                        triangles.push(TriangleObj {
                            p0: lines[i * 3] as i32,
                            p1: lines[i * 3 + 1] as i32,
                            p2: lines[i * 3 + 2] as i32,
                        });
                    }
                    index_data.append(&mut lines);
                }
            }
        }
        let uniform_buf = BufferObj::create_uniform_buffer(
            &app_view.device,
            &ClothUniform {
                num_x: particle_x_num as i32,
                num_y: particle_y_num as i32,
                triangle_num: triangles.len() as i32,
                compliance: 0.000 / (0.016 * 0.016),
                dt: 0.016,
            },
            None,
        );
        let triangle_buf = BufferObj::create_storage_buffer(
            &app_view.device,
            triangles.as_bytes(),
            Some("triangle_buf"),
        );
        let collision_constraint_buf = BufferObj::create_empty_storage_buffer(
            &app_view.device,
            (particle_num * 4 * (4 + 8 + 4 * 8)) as wgpu::BufferAddress,
            false,
            Some("collision_constraint_buf"),
        );

        // 空间 hash 网格
        let bin_num_x = (factor.0 * 2.0 / average_edge).ceil() as i32;
        let bin_num: [i32; 4] =
            [bin_num_x, (factor.1 * 2.0 / average_edge).ceil() as i32, bin_num_x, 0];
        let max_bin_count = bin_num[0] * bin_num[1] * bin_num[2];
        let bin_uniform_buf = BufferObj::create_uniform_buffer(
            &app_view.device,
            &BinUniform {
                bin_num,
                bin_max_index: [bin_num[0] - 1, bin_num[1] - 1, bin_num[2] - 1, 0],
                bin_size: [average_edge, average_edge, average_edge, 0.0],
                pos_offset: [factor.0, factor.1, factor.0, 0.0],
                max_bin_count,
                padding: [0.0; 3],
            },
            None,
        );
        // 空间 hash 网格 buffer
        let bin_buf_size = max_bin_count * 4 * 16;
        let bin_buf = BufferObj::create_empty_storage_buffer(
            &app_view.device,
            bin_buf_size as wgpu::BufferAddress,
            false,
            Some("bin_buf"),
        );

        let particle_buf = BufferObj::create_storage_buffer(
            &app_view.device,
            particles.as_bytes(),
            Some("particle_buf"),
        );
        let particle_constraints_buf = BufferObj::create_storage_buffer(
            &app_view.device,
            particle_constraints.as_bytes(),
            Some("constraint_buf"),
        );

        let stretch_constraints_buf = BufferObj::create_storage_buffer(
            &app_view.device,
            stretch_constraints.as_bytes(),
            Some("stretch_constraints_buf"),
        );
        let bending_constraints_buf = BufferObj::create_storage_buffer(
            &app_view.device,
            bending_constraints.as_bytes(),
            Some("bending_constraints_buf"),
        );
        let mesh_coloring_buf = BufferObj::create_storage_buffer(
            &app_view.device,
            mesh_coloring_buf_data.as_bytes(),
            Some("mesh_coloring_buf"),
        );

        let predict_and_reset_shader = idroid::shader::create_shader_module(
            &app_view.device,
            "pbd/cloth_predict_and_reset2",
            None,
        );

        // 调试用的 buffer
        let size = triangles.len() * 4 * 16;
        let debug_buf = BufferObj::create_empty_storage_buffer(
            &app_view.device,
            size as wgpu::BufferAddress,
            false,
            Some("debug_buf"),
        );
        let inout_buffers: Vec<&BufferObj> = vec![
            &particle_buf,
            &bin_buf,
            &particle_constraints_buf,
            &stretch_constraints_buf,
            &bending_constraints_buf,
            &mesh_coloring_buf,
            &triangle_buf,
            &collision_constraint_buf,
            // &debug_buf,
        ];
        let predict_and_reset = ComputeNode::new(
            &app_view.device,
            ((particle_x_num + 15) / 16, (particle_y_num + 15) / 16, 1),
            vec![&uniform_buf, &bin_uniform_buf, &frame_uniform_buf],
            inout_buffers.clone(),
            vec![],
            &predict_and_reset_shader,
        );

        let binning_shader =
            idroid::shader::create_shader_module(&app_view.device, "pbd/cloth_binning", None);

        let binning_node = ComputeNode::new(
            &app_view.device,
            ((particle_x_num + 15) / 16, (particle_y_num + 15) / 16, 1),
            vec![&uniform_buf, &bin_uniform_buf],
            inout_buffers.clone(),
            vec![],
            &binning_shader,
        );

        let gen_collision_shader =
            idroid::shader::create_shader_module(&app_view.device, "pbd/cloth_gen_collision", None);

        let gen_collision = ComputeNode::new(
            &app_view.device,
            ((triangles.len() as u32 + 31) / 32, 1, 1),
            vec![&uniform_buf, &bin_uniform_buf],
            inout_buffers.clone(),
            vec![],
            &gen_collision_shader,
        );

        let constraints_solver_shader = idroid::shader::create_shader_module(
            &app_view.device,
            "pbd/cloth_constraints_solver",
            None,
        );
        let constraints_solver = ComputeNode::new(
            &app_view.device,
            (0, 0, 0),
            vec![&uniform_buf, &bin_uniform_buf],
            inout_buffers.clone(),
            vec![],
            &constraints_solver_shader,
        );

        let base_path = if cfg!(target_os = "macos") {
            env!("CARGO_MANIFEST_DIR").to_string()
        } else {
            FileSystem::get_bundle_url().to_string() + "/"
        };
        // 1863*3312
        // let img_path = PathBuf::from(&base_path).join("assets/paper/3.png");
        let (texture, _sampler) = idroid::load_texture::from_path(
            "2021_fu/dragon.png",
            app_view,
            wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
            false,
        );
        let display_shader =
            idroid::shader::create_shader_module(&app_view.device, "pbd/cloth_display", None);
        let sampler = idroid::load_texture::bilinear_sampler(&app_view.device);
        let display_node_builder =
            ViewNodeBuilder::<PosParticleIndex>::new(vec![(&texture, None)], &display_shader)
                .with_uniform_buffers(vec![&mvp_buf, &uniform_buf])
                .with_storage_buffers(vec![&particle_buf, &collision_constraint_buf])
                .with_use_depth_stencil(true)
                .with_samplers(vec![&sampler])
                .with_shader_stages(vec![
                    wgpu::ShaderStages::VERTEX,
                    wgpu::ShaderStages::VERTEX,
                    wgpu::ShaderStages::VERTEX,
                    wgpu::ShaderStages::VERTEX,
                    wgpu::ShaderStages::FRAGMENT,
                    wgpu::ShaderStages::FRAGMENT,
                ])
                .with_vertices_and_indices((vertex_data.clone(), index_data));

        let display_node = display_node_builder.build(&app_view.device);

        let point_shader =
            idroid::shader::create_shader_module(&app_view.device, "pbd/cloth_point_display", None);
        let point_node_builder = ViewNodeBuilder::<PosParticleIndex>::new(vec![], &point_shader)
            .with_uniform_buffers(vec![&mvp_buf, &uniform_buf])
            .with_storage_buffers(vec![&particle_buf, &collision_constraint_buf])
            .with_use_depth_stencil(true)
            .with_shader_stages(vec![
                wgpu::ShaderStages::VERTEX,
                wgpu::ShaderStages::VERTEX,
                wgpu::ShaderStages::VERTEX,
                wgpu::ShaderStages::VERTEX,
            ])
            .with_primitive_topology(wgpu::PrimitiveTopology::PointList)
            .with_vertices_and_indices((vertex_data, point_index_data));

        let point_node = point_node_builder.build(&app_view.device);

        let normal_shader = idroid::shader::create_shader_module(
            &app_view.device,
            "pbd/cloth_normal_display",
            None,
        );
        let normal_node_builder = ViewNodeBuilder::<PosParticleIndex>::new(vec![], &normal_shader)
            .with_uniform_buffers(vec![&mvp_buf, &uniform_buf])
            .with_storage_buffers(vec![&particle_buf, &collision_constraint_buf, &triangle_buf])
            .with_use_depth_stencil(true)
            .with_shader_stages(vec![
                wgpu::ShaderStages::VERTEX,
                wgpu::ShaderStages::VERTEX,
                wgpu::ShaderStages::VERTEX,
                wgpu::ShaderStages::VERTEX,
                wgpu::ShaderStages::VERTEX,
            ])
            .with_primitive_topology(wgpu::PrimitiveTopology::LineList)
            .with_vertices_and_indices((normal_data, normal_index_data));

        let normal_line_node = normal_node_builder.build(&app_view.device);

        // 初始化为卷起的状态
        let half_height = tl_y;
        let start_radius = 50.0 * pixel_on_ndc;
        let roll_length = half_height * 2.0 - start_radius * 3.0;
        // let roll_length = start_radius * 6.1;
        let roll_to = roll_length;
        let roll_uniform_buf = BufferObj::create_uniforms_buffer(
            &app_view.device,
            &vec![roll_to, roll_length, start_radius, half_height],
            None,
        );
        let roll_shader =
            idroid::shader::create_shader_module(&app_view.device, "pbd/cloth_roll_init", None);
        let roll_init = ComputeNode::new(
            &app_view.device,
            ((particle_x_num + 15) / 16, (particle_y_num + 15) / 16, 1),
            vec![&uniform_buf, &roll_uniform_buf],
            vec![&particle_buf],
            vec![],
            &roll_shader,
        );
        roll_init.compute(&mut encoder);

        println!("pixel on ndc: {}", pixel_on_ndc);

        let size = wgpu::Extent3d {
            width: app_view.config.width,
            height: app_view.config.height,
            depth_or_array_layers: 1,
        };
        let depth_texture_view =
            idroid::depth_stencil::create_depth_texture_view(size, &app_view.device);

        let mut instance = Self {
            frame_uniform_buf,
            mesh_coloring,
            predict_and_reset,
            binning_node,
            gen_collision,
            constraints_solver,
            display_node,
            point_node,
            normal_line_node,
            depth_texture_view,
            frame_count: 0,
        };

        instance.step_solver(&mut encoder);
        app_view.queue.submit(Some(encoder.finish()));

        instance
    }

    fn step_solver(&mut self, encoder: &mut wgpu::CommandEncoder) {
        // 重用 cpass 在 macOS 上不能提升性能， 但是在 iOS 上提升明显
        // 64*64，8 约束，迭代20 ：Xs Max, 12ms -> 8ms
        let mut cpass =
            encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("solver pass") });
        cpass.set_pipeline(&self.predict_and_reset.pipeline);
        cpass.set_bind_group(0, &self.predict_and_reset.bg_setting.bind_group, &[]);
        cpass.dispatch(
            self.predict_and_reset.group_count.0,
            self.predict_and_reset.group_count.1,
            1,
        );

        cpass.set_pipeline(&self.binning_node.pipeline);
        cpass.set_bind_group(0, &self.binning_node.bg_setting.bind_group, &[]);
        cpass.dispatch(self.binning_node.group_count.0, self.binning_node.group_count.1, 1);

        cpass.set_pipeline(&self.gen_collision.pipeline);
        cpass.set_bind_group(0, &self.gen_collision.bg_setting.bind_group, &[]);
        cpass.dispatch(self.gen_collision.group_count.0, self.gen_collision.group_count.1, 1);

        // self.binning_node.dispatch(&mut cpass);
        // self.gen_collision.dispatch(&mut cpass);

        for i in 0..15 {
            cpass.set_pipeline(&self.constraints_solver.pipeline);
            cpass.set_bind_group(0, &self.constraints_solver.bg_setting.bind_group, &[]);
            for mc in self.mesh_coloring.iter() {
                // println!("{:?}", mc.get_push_constants_data());
                cpass.set_push_constants(0, mc.get_bending_push_constants_data(i).as_bytes());
                cpass.dispatch(mc.thread_group.0, mc.thread_group.1, 1);
            }
        }

        self.frame_count += 1;
    }

    pub fn enter_frame(&mut self, app_view: &mut AppView) {
        let mut encoder = app_view.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("cloth encoder"),
        });
        self.step_solver(&mut encoder);

        let (frame, frame_view) = app_view.get_current_frame_view();

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("cloth render pass"),
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
            // self.point_node.draw_render_pass(&mut rpass);
            // self.normal_line_node.draw_render_pass(&mut rpass);
        }
        app_view.queue.submit(Some(encoder.finish()));
        frame.present();

        app_view.queue.write_buffer(
            &self.frame_uniform_buf.buffer,
            0,
            &FrameUniform { frame_index: self.frame_count as i32 }.as_bytes(),
        );
    }
}
