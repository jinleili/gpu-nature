use crate::{setting_obj::SettingObj, FieldUniform, Player};
use idroid::node::{BufferlessFullscreenNode, ComputeNode};
use idroid::{math::Size, BufferObj};
use wgpu::{CommandEncoderDescriptor, Device, Queue};

use crate::{create_shader_module, insert_code_then_create};

pub struct FieldPlayer {
    canvas_size: Size<u32>,
    field_uniform_data: FieldUniform,
    field_uniform: BufferObj,
    field_buf: BufferObj,
    trajectory_update_shader: wgpu::ShaderModule,
    field_setting_node: ComputeNode,
    particles_update_node: ComputeNode,
    render_node: BufferlessFullscreenNode,
    frame_num: usize,
}

impl FieldPlayer {
    pub fn new(
        device: &wgpu::Device, queue: &wgpu::Queue, canvas_format: wgpu::TextureFormat,
        canvas_size: Size<u32>, canvas_buf: &BufferObj, setting: &SettingObj,
    ) -> Self {
        let pixel_distance = 4;
        let field_size: idroid::math::Size<u32> =
            (canvas_size.width / pixel_distance, canvas_size.height / pixel_distance).into();
        println!("field_size: {:?} \n {:?}", field_size, canvas_size);

        let field_threadgroup = ((field_size.width + 15) / 16, (field_size.height + 15) / 16, 1);
        let (_, sx, sy) = idroid::utils::matrix_helper::fullscreen_factor(
            (canvas_size.width as f32, canvas_size.height as f32).into(),
        );
        let field_uniform_data = FieldUniform {
            lattice_size: [field_size.width as i32, field_size.height as i32, 1, 0],
            lattice_pixel_size: [pixel_distance as f32; 4],
            canvas_size: [canvas_size.width as i32, canvas_size.height as i32, 1, 0],
            normalized_space_size: [sx, sy, 0.0, 0.0],
            pixel_distance: [pixel_distance as f32; 4],
            speed_ty: 0,
            _padding: [0.0; 3],
        };
        let field_uniform =
            BufferObj::create_uniform_buffer(device, &field_uniform_data, Some("field_uniform"));
        let field_buf = BufferObj::create_empty_storage_buffer(
            device,
            (field_size.width * field_size.height * 16) as u64,
            false,
            Some("field buf"),
        );

        let code_segment = crate::get_velocity_code_segment(setting.animation_type);

        let setting_shader =
            insert_code_then_create(device, "field_setting", Some(code_segment), None);

        let field_setting_node = ComputeNode::new(
            device,
            field_threadgroup,
            vec![&field_uniform],
            vec![&field_buf],
            vec![],
            &setting_shader,
        );

        let trajectory_update_shader = create_shader_module(device, "trajectory_update", None);
        let particles_update_node = ComputeNode::new(
            device,
            setting.particles_threadgroup,
            vec![&field_uniform, &setting.particles_uniform.as_ref().unwrap()],
            vec![&field_buf, &setting.particles_buf.as_ref().unwrap(), canvas_buf],
            vec![],
            &trajectory_update_shader,
        );

        let render_shader = create_shader_module(device, "present", None);
        let render_node = BufferlessFullscreenNode::new(
            device,
            canvas_format,
            vec![&field_uniform, &setting.particles_uniform.as_ref().unwrap()],
            vec![canvas_buf],
            vec![],
            vec![],
            None,
            &render_shader,
        );
        let mut instance = FieldPlayer {
            canvas_size,
            field_uniform_data,
            field_uniform,
            field_buf,
            trajectory_update_shader,
            field_setting_node,
            particles_update_node,
            render_node,
            frame_num: 0,
        };
        instance
    }

    pub fn update_field_by_cpass<'c, 'b: 'c>(&'b self, cpass: &mut wgpu::ComputePass<'c>) {
        self.field_setting_node.dispatch(cpass);
    }
}

impl Player for FieldPlayer {
    fn reset(&mut self, device: &Device, queue: &Queue) {
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("update_field encoder"),
        });
        self.field_setting_node.compute(&mut encoder);
        queue.submit(Some(encoder.finish()));
    }

    fn enter_frame(
        &mut self, device: &Device, queue: &Queue, frame_view: &wgpu::TextureView,
        _setting: &mut crate::SettingObj,
    ) {
        //  On latast wgpu(2021/06/05), must reset twice to get correct result
        if self.frame_num <= 1 {
            self.reset(device, queue);
        }
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("field player encoder"),
        });
        self.particles_update_node.compute(&mut encoder);
        self.render_node.draw(
            frame_view,
            &mut encoder,
            wgpu::LoadOp::Clear(wgpu::Color { r: 0.1, g: 0.15, b: 0.17, a: 1.0 }),
        );
        queue.submit(Some(encoder.finish()));
        self.frame_num += 1;
    }
}
