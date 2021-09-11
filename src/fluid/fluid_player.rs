use super::{AAD2Q9Node, D2Q9Node, ParticleRenderNode, OBSTACLE_RADIUS};
use crate::{fluid::LbmUniform, setting_obj::SettingObj, FieldAnimationType, Player};
use idroid::{
    math::{Position, Size},
    node::{BufferlessFullscreenNode, ComputeNode},
    BufferObj,
};
use wgpu::{CommandEncoderDescriptor, Device, Queue, TextureFormat};
use zerocopy::AsBytes;

use crate::create_shader_module;

// 通用的流體模擬，產生外部依賴的流體量
pub struct FluidPlayer {
    animation_ty: FieldAnimationType,
    canvas_size: Size<u32>,
    lattice: wgpu::Extent3d,
    lattice_pixel_size: u32,
    pre_pos: Position,
    fluid_compute_node: AAD2Q9Node,
    // collide scheme
    use_aa_pattern: bool,
    curl_cal_node: ComputeNode,
    particle_update_node: ComputeNode,
    render_node: BufferlessFullscreenNode,
    particle_render: BufferlessFullscreenNode,
}

impl FluidPlayer {
    pub fn new(
        app_view: &idroid::AppView, canvas_size: Size<u32>, canvas_buf: &BufferObj,
        setting: &SettingObj,
    ) -> Self {
        let device = &app_view.device;
        let use_aa_pattern = true;
        let fluid_compute_node = AAD2Q9Node::new(app_view, canvas_size, setting);
        let lattice = fluid_compute_node.lattice;

        let curl_shader =
            create_shader_module(device, "lbm/curl_update", Some("curl_update_shader"));
        let curl_texture_format = TextureFormat::R32Float;
        let curl_tex = idroid::load_texture::empty(
            device,
            curl_texture_format,
            wgpu::Extent3d {
                width: lattice.width,
                height: lattice.height,
                depth_or_array_layers: 1,
            },
            None,
            Some(wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING),
            Some("curl_tex"),
        );
        let curl_cal_node = ComputeNode::new(
            device,
            fluid_compute_node.dispatch_group_count,
            vec![&fluid_compute_node.lbm_uniform_buf, &fluid_compute_node.fluid_uniform_buf],
            vec![&fluid_compute_node.info_buf],
            vec![
                (&fluid_compute_node.macro_tex, None),
                (&curl_tex, Some(wgpu::StorageTextureAccess::WriteOnly)),
            ],
            &curl_shader,
        );

        let render_shader = create_shader_module(device, "lbm/present", Some("lbm present shader"));
        let sampler = idroid::load_texture::bilinear_sampler(device);
        let render_node = BufferlessFullscreenNode::new(
            device,
            app_view.config.format,
            vec![
                &fluid_compute_node.fluid_uniform_buf,
                &setting.particles_uniform.as_ref().unwrap(),
            ],
            vec![&canvas_buf],
            vec![(&fluid_compute_node.macro_tex, None), (&curl_tex, None)],
            vec![&sampler],
            None,
            &render_shader,
        );

        let update_shader =
            create_shader_module(device, "lbm/particle_update", Some("particle_update_shader"));
        let particle_update_node = ComputeNode::new(
            device,
            setting.particles_threadgroup,
            vec![
                &fluid_compute_node.lbm_uniform_buf,
                &fluid_compute_node.fluid_uniform_buf,
                &setting.particles_uniform.as_ref().unwrap(),
            ],
            vec![&setting.particles_buf.as_ref().unwrap(), &canvas_buf],
            vec![(&fluid_compute_node.macro_tex, None)],
            &update_shader,
        );

        let particle_shader = create_shader_module(device, "present", None);
        let particle_render = BufferlessFullscreenNode::new(
            device,
            app_view.config.format,
            vec![
                &fluid_compute_node.fluid_uniform_buf,
                &setting.particles_uniform.as_ref().unwrap(),
            ],
            vec![&canvas_buf],
            vec![],
            vec![],
            None,
            &particle_shader,
        );

        FluidPlayer {
            animation_ty: setting.animation_type,
            canvas_size,
            lattice,
            use_aa_pattern,
            lattice_pixel_size: fluid_compute_node.lattice_pixel_size,
            pre_pos: Position::new(0.0, 0.0),
            fluid_compute_node,
            curl_cal_node,
            particle_update_node,
            render_node,
            particle_render,
        }
    }
}

impl Player for FluidPlayer {
    fn on_click(
        &mut self, _device: &wgpu::Device, queue: &wgpu::Queue, pos: idroid::math::Position,
    ) {
        if pos.x <= 0.0 || pos.y <= 0.0 {
            return;
        }
        let x = pos.x as u32 / self.lattice_pixel_size;
        let y = pos.y as u32 / self.lattice_pixel_size;
        let half_size = OBSTACLE_RADIUS as u32;
        if x < half_size
            || x >= self.lattice.width - (half_size + 2)
            || y < half_size
            || y >= self.lattice.height - (half_size + 2)
        {
            return;
        }
        self.fluid_compute_node.add_obstacle(queue, x, y);
    }

    fn touch_begin(&mut self, _device: &wgpu::Device, _queue: &wgpu::Queue) {
        self.pre_pos = Position::new(0.0, 0.0);
    }

    fn touch_move(&mut self, _device: &Device, queue: &Queue, pos: Position) {
        if pos.x <= 0.0 || pos.y <= 0.0 {
            self.pre_pos = Position::zero();
            return;
        }
        let dis = pos.distance(&self.pre_pos);
        if (self.pre_pos.x == 0.0 && self.pre_pos.y == 0.0) || dis > 300.0 {
            self.pre_pos = pos;
            return;
        }

        self.fluid_compute_node.add_external_force(queue, pos, self.pre_pos);

        self.pre_pos = pos;
    }

    fn update_uniforms(&mut self, queue: &Queue, setting: &crate::SettingObj) {
        // 通过外部参数来重置流体粒子碰撞松解时间 tau = (3.0 * x + 0.5), x：[0~1] 趋大，松解时间趋快
        let tau = 3.0 * setting.fluid_viscosity + 0.5;
        let fluid_ty = if setting.animation_type == FieldAnimationType::Poiseuille { 0 } else { 1 };
        let uniform_data =
            LbmUniform::new(tau, fluid_ty, (self.lattice.width * self.lattice.height) as i32);
        queue.write_buffer(
            &self.fluid_compute_node.lbm_uniform_buf.buffer,
            0,
            uniform_data.as_bytes(),
        );
    }

    fn reset(&mut self, device: &Device, queue: &Queue) {
        self.fluid_compute_node.reset_lattice_info(device, queue);

        self.pre_pos = Position::new(0.0, 0.0);
    }

    fn enter_frame(
        &mut self, device: &Device, queue: &Queue, frame_view: &wgpu::TextureView,
        setting: &mut crate::SettingObj,
    ) {
        setting.particles_uniform_data.is_only_update_pos = 1;
        setting.update_particles_uniform(queue);
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("fluid player encoder"),
        });
        {
            let mut cpass =
                encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("fluid solver") });

            for _ in 0..3 {
                self.fluid_compute_node.dispatch(&mut cpass, 0);
                self.particle_update_node.dispatch(&mut cpass);
                // self.curl_cal_node.dispatch(&mut cpass);

                if !self.use_aa_pattern {
                    self.fluid_compute_node.dispatch(&mut cpass, 1);
                    self.particle_update_node.dispatch(&mut cpass);
                    // self.curl_cal_node.dispatch(&mut cpass);
                }
            }
        }
        // draw macro_tex
        {
            // let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            //     label: None,
            //     color_attachments: &[wgpu::RenderPassColorAttachment {
            //         view: frame_view,
            //         resolve_target: None,
            //         ops: wgpu::Operations {
            //             load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.2, g: 0.2, b: 0.25, a: 1.0 }),
            //             store: true,
            //         },
            //     }],
            //     depth_stencil_attachment: None,
            // });
            // self.render_node.draw_rpass(&mut rpass);
        }
        // draw paticles
        {
            self.particle_render.draw(
                frame_view,
                &mut encoder,
                wgpu::LoadOp::Clear(wgpu::Color { r: 0.2, g: 0.2, b: 0.25, a: 1.0 }),
            );
        }
        queue.submit(Some(encoder.finish()));
    }
}
