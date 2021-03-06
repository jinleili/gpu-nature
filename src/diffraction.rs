use crate::util::node::{ViewNode, ViewNodeBuilder};
use crate::util::BufferObj;

use app_surface::{
    math::{Position, Size},
    AppSurface, SurfaceFrame, Touch, TouchPhase,
};
use nalgebra_glm as glm;
use zerocopy::AsBytes;

pub struct Diffraction {
    mvp_buf: BufferObj,
    translate_z: f32,
    proj_mat: glm::TMat4<f32>,
    disc_inner_circle: ViewNode,
    diffraction_node: ViewNode,
}

impl Diffraction {
    pub fn new(app_view: &AppSurface, is_use_depth_stencil: bool) -> Self {
        // let mut encoder =
        //     app_view.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        let viewport_size: Size<f32> = (&app_view.config).into();
        let (proj_mat, mv_mat, factor) =
            crate::util::utils::matrix_helper::perspective_mvp(viewport_size);
        // change mv_mat's z to 0
        let translate_z = factor.2 - 0.6;

        let mvp_uniform = crate::MVPMatUniform {
            mv: mv_mat.into(),
            proj: proj_mat.into(),
            mvp: (proj_mat * mv_mat).into(),
            normal: mv_mat.into(),
        };
        let mvp_buf = BufferObj::create_uniform_buffer(&app_view.device, &mvp_uniform, None);
        let light_x: f32 = -0.0;
        let light_y: f32 = -5.0;
        let light_z: f32 = 10.0;
        // distance between adjacent tracks: CD 1600nm, DVD 740nm
        let d: f32 = 1240.0;
        let uniform_buf = BufferObj::create_uniform_buffer(
            &app_view.device,
            &[light_x, light_y, light_z, d],
            None,
        );

        let (vertex_data, index_data) = crate::generate_disc_plane(0.255, 0.99, 360);
        let (inner_circle_vertex_data, inner_circle_index_data) =
            crate::generate_disc_plane(0.1, 0.26, 50);

        let diffraction_shader =
            crate::util::shader::create_shader_module(&app_view.device, "diffraction_vertex", None);
        let simle_shader =
            crate::util::shader::create_shader_module(&app_view.device, "simple", None);

        let builder = ViewNodeBuilder::<crate::PosTangent>::new(vec![], &diffraction_shader)
            .with_uniform_buffers(vec![&mvp_buf, &uniform_buf])
            .with_primitive_topology(wgpu::PrimitiveTopology::TriangleList)
            .with_vertices_and_indices((vertex_data, index_data))
            .with_shader_stages(vec![wgpu::ShaderStages::VERTEX, wgpu::ShaderStages::VERTEX])
            .with_color_format(app_view.config.format)
            .with_use_depth_stencil(is_use_depth_stencil);
        let diffraction_node = builder.build(&app_view.device);

        let inner_circle_builder = ViewNodeBuilder::<crate::PosTangent>::new(vec![], &simle_shader)
            .with_uniform_buffers(vec![&mvp_buf])
            .with_primitive_topology(wgpu::PrimitiveTopology::TriangleList)
            .with_vertices_and_indices((inner_circle_vertex_data, inner_circle_index_data))
            .with_shader_stages(vec![wgpu::ShaderStages::VERTEX])
            .with_color_format(app_view.config.format)
            .with_use_depth_stencil(is_use_depth_stencil);
        let disc_inner_circle = inner_circle_builder.build(&app_view.device);
        let mut instance =
            Self { proj_mat, translate_z, mvp_buf, diffraction_node, disc_inner_circle };
        instance.rotate(app_view, -0.3, -0.4);
        instance
    }

    pub fn rotate(&mut self, app_view: &app_surface::AppSurface, x: f32, y: f32) {
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

    pub fn enter_frame(&mut self, app_view: &mut AppSurface) {
        let (frame, frame_view) = app_view.get_current_frame_view();
        let color_attachments = [Some(wgpu::RenderPassColorAttachment {
            view: &frame_view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(crate::util::utils::alpha_color()),
                store: true,
            },
        })];
        let render_pass_descriptor = wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &color_attachments,
            depth_stencil_attachment: None,
        };

        let mut encoder = app_view.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("diffraction encoder"),
        });
        {
            let mut rpass = encoder.begin_render_pass(&render_pass_descriptor);
            self.draw_render_pass(&mut rpass);
        }
        app_view.queue.submit(Some(encoder.finish()));
        frame.present();
    }

    pub fn draw_render_pass<'a, 'b: 'a>(&'b self, rpass: &mut wgpu::RenderPass<'b>) {
        self.diffraction_node.draw_render_pass(rpass);
        self.disc_inner_circle.draw_render_pass(rpass);
    }
}
