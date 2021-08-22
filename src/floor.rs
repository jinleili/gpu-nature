use crate::noise::{create_gradient_buf, create_permulation_buf};
use idroid::node::{BufferlessFullscreenNode, ComputeNode, ImageNodeBuilder, ImageViewNode};
use nalgebra_glm as glm;

pub struct Floor {
    display_node: ImageViewNode,
    noise_display: BufferlessFullscreenNode,
}

impl Floor {
    pub fn new(
        app_view: &idroid::AppView, noise_tex: &idroid::AnyTexture, is_use_depth_stencil: bool,
    ) -> Floor {
        let threadgroup_count = (62, 62, 1);
        let tex_width = 16 * 62;
        let marble_tex = idroid::load_texture::empty(
            &app_view.device,
            wgpu::TextureFormat::Rgba8Unorm,
            wgpu::Extent3d { width: tex_width, height: tex_width, depth_or_array_layers: 1 },
            None,
            Some(wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING),
        );
        let permulation_buf = create_permulation_buf(&app_view.device);
        let gradient_buf = create_gradient_buf(&app_view.device);
        let marble_shader =
            idroid::shader::create_shader_module(&app_view.device, "noise/marble_tex", None);
        let marble_node = ComputeNode::new(
            &app_view.device,
            threadgroup_count,
            vec![],
            vec![&permulation_buf, &gradient_buf],
            vec![(&marble_tex, Some(wgpu::StorageTextureAccess::WriteOnly))],
            &marble_shader,
        );

        let mut encoder = app_view.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("diffraction encoder"),
        });
        marble_node.compute(&mut encoder);
        app_view.queue.submit(Some(encoder.finish()));

        let (p_matrix, mut mv_matrix, factor) =
            idroid::utils::matrix_helper::perspective_mvp((&app_view.config).into());
        // mv_matrix = glm::translate(&mv_matrix, &glm::vec3(0.0, -1.1 * factor.1, -1.1));
        // let scale = factor.0 * 1.2;
        // mv_matrix = glm::scale(&mv_matrix, &glm::vec3(scale, scale, 1.0));
        // mv_matrix = glm::rotate_x(&mv_matrix, -1.57);

        let rotate_mat = glm::rotate_x(&glm::TMat4::<f32>::identity(), -1.57);
        let translate_mat =
            glm::translate(&glm::TMat4::<f32>::identity(), &glm::vec3(0.0, -1.0, -2.0));
        let new_mv_mat = translate_mat * rotate_mat;

        let normal: [[f32; 4]; 4] = glm::inverse_transpose(new_mv_mat).into();
        let mvp_uniform = crate::MVPMatUniform {
            mv: new_mv_mat.into(),
            proj: p_matrix.into(),
            mvp: (p_matrix * new_mv_mat).into(),
            normal: normal,
        };
        let mvp_buf =
            idroid::BufferObj::create_uniform_buffer(&app_view.device, &mvp_uniform, None);
        let (vertices, indices) = idroid::geometry::Plane::new(1, 1).generate_vertices();
        let floor_shader = idroid::shader::create_shader_module(&app_view.device, "floor", None);
        let default_sampler = idroid::load_texture::default_sampler(&app_view.device);
        let mirror_sampler = idroid::load_texture::mirror_repeate_sampler(&app_view.device);
        let builder = ImageNodeBuilder::<idroid::vertex::PosTex>::new(
            vec![(&marble_tex, None), (noise_tex, None)],
            &floor_shader,
        )
        .with_samplers(vec![&default_sampler, &mirror_sampler])
        .with_uniform_buffers(vec![&mvp_buf])
        .with_vertices_and_indices((vertices, indices))
        .with_primitive_topology(wgpu::PrimitiveTopology::TriangleList)
        .with_shader_stages(vec![
            wgpu::ShaderStages::VERTEX,
            wgpu::ShaderStages::FRAGMENT,
            wgpu::ShaderStages::FRAGMENT,
            wgpu::ShaderStages::FRAGMENT,
            wgpu::ShaderStages::FRAGMENT,
        ])
        .with_color_format(app_view.config.format)
        .with_use_depth_stencil(is_use_depth_stencil);
        let display_node = builder.build(&app_view.device);

        let noise_shader =
            idroid::shader::create_shader_module(&app_view.device, "noise/perlin_noise", None);

        let noise_display = BufferlessFullscreenNode::new(
            &app_view.device,
            app_view.config.format,
            vec![],
            vec![&permulation_buf, &gradient_buf],
            vec![],
            vec![],
            Some(vec![wgpu::ShaderStages::FRAGMENT, wgpu::ShaderStages::FRAGMENT]),
            &noise_shader,
        );
        Floor { display_node, noise_display }
    }

    pub fn draw_render_pass<'a, 'b: 'a>(&'b self, rpass: &mut wgpu::RenderPass<'b>) {
        self.display_node.draw_render_pass(rpass);
    }
}
