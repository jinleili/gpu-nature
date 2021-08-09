use idroid::node::BindingGroupSettingNode;
use idroid::node::{ImageNodeBuilder, ImageViewNode};
use idroid::vertex::{Pos, PosOnly};
use idroid::{math::Size, vertex::PosParticleIndex, BufferObj};

use nalgebra_glm as glm;
use uni_view::{fs::FileSystem, AppView, GPUContext};
use zerocopy::AsBytes;

pub struct Diffraction {
    disc_inner_circle: ImageViewNode,
    diffraction_node: ImageViewNode,
}

impl Diffraction {
    pub fn new(app_view: &AppView) -> Self {
        // let mut encoder =
        //     app_view.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        let viewport_size: Size<f32> = (&app_view.sc_desc).into();
        let (p_matrix, mut mv_matrix, factor) =
            idroid::utils::matrix_helper::perspective_mvp(viewport_size);
        let mut model_rotate_mat = glm::TMat4::<f32>::identity();
        model_rotate_mat = glm::translate(&model_rotate_mat, &glm::TVec3::new(0.0, 0.0, -0.6));
        model_rotate_mat = glm::rotate_x(&model_rotate_mat, -0.65);
        mv_matrix = mv_matrix * model_rotate_mat;

        let normal: [[f32; 4]; 4] = glm::inverse_transpose(mv_matrix).into();
        let mvp_uniform = crate::MVPMatUniform {
            mv: mv_matrix.into(),
            proj: p_matrix.into(),
            mvp: (p_matrix * mv_matrix).into(),
            normal: normal,
            // normal: mv_matrix.into()
        };
        let mvp_buf = BufferObj::create_uniform_buffer(&app_view.device, &mvp_uniform, None);
        let light_x: f32 = -0.0;
        let light_y: f32 = -5.0;
        let light_z: f32 = 10.0;
        // distance between adjacent tracks: CD 1600nm, DVD 740nm
        let d: f32 = 1600.0;
        let uniform_buf = BufferObj::create_uniform_buffer(
            &app_view.device,
            &[light_x, light_y, light_z, d],
            None,
        );

        let (vertex_data, index_data) = crate::generate_disc_plane(0.255, 0.99, 150);
        let (inner_circle_vertex_data, inner_circle_index_data) =
            crate::generate_disc_plane(0.1, 0.26, 50);

        let diffraction_shader =
            idroid::shader::create_shader_module(&app_view.device, "diffraction", None);
        let simle_shader = idroid::shader::create_shader_module(&app_view.device, "simple", None);

        let builder = ImageNodeBuilder::<crate::PosTangent>::new(vec![], &diffraction_shader)
            .with_uniform_buffers(vec![&mvp_buf, &uniform_buf])
            .with_primitive_topology(wgpu::PrimitiveTopology::TriangleList)
            .with_vertices_and_indices((vertex_data, index_data))
            .with_shader_states(vec![wgpu::ShaderStages::VERTEX, wgpu::ShaderStages::FRAGMENT]);
        let diffraction_node = builder.build(&app_view.device);

        let inner_circle_builder =
            ImageNodeBuilder::<crate::PosTangent>::new(vec![], &simle_shader)
                .with_uniform_buffers(vec![&mvp_buf])
                .with_primitive_topology(wgpu::PrimitiveTopology::TriangleList)
                .with_vertices_and_indices((inner_circle_vertex_data, inner_circle_index_data))
                .with_shader_states(vec![wgpu::ShaderStages::VERTEX]);
        let disc_inner_circle = inner_circle_builder.build(&app_view.device);
        Self { diffraction_node, disc_inner_circle }
    }
    pub fn enter_frame(&mut self, app_view: &mut AppView) {
        let mut encoder = app_view.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("diffraction encoder"),
        });
        let frame = match app_view.swap_chain.get_current_frame() {
            Ok(frame) => frame,
            Err(_) => {
                app_view.update_swap_chain();
                app_view
                    .swap_chain
                    .get_current_frame()
                    .expect("Failed to acquire next swap chain texture!")
            }
        };
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &frame.output.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(idroid::utils::alpha_color()),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });
            self.diffraction_node.draw_render_pass(&mut rpass);
            self.disc_inner_circle.draw_render_pass(&mut rpass);
        }
        app_view.queue.submit(Some(encoder.finish()));
    }
}
