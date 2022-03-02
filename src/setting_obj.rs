use crate::util::BufferObj;
use crate::{
    get_particles_data, FieldAnimationType, FieldType, ParticleColorType, ParticleUniform,
};
use app_surface::math::Size;
use zerocopy::AsBytes;

pub struct SettingObj {
    canvas_size: Size<u32>,
    pub field_type: FieldType,
    pub animation_type: FieldAnimationType,
    pub color_ty: ParticleColorType,
    pub fluid_viscosity: f32,

    pub particles_count: i32,
    pub particle_lifetime: f32,
    pub particles_uniform_data: ParticleUniform,
    pub particles_uniform: Option<BufferObj>,
    pub particles_buf: Option<BufferObj>,
    pub particles_size: wgpu::Extent3d,
    pub particles_threadgroup: (u32, u32, u32),
}

impl SettingObj {
    pub fn new(
        field_type: FieldType, animation_type: FieldAnimationType, color_ty: ParticleColorType,
        particles_count: i32, particle_lifetime: f32,
    ) -> Self {
        SettingObj {
            canvas_size: (0_u32, 0_u32).into(),
            field_type,
            animation_type,
            fluid_viscosity: 0.02,
            color_ty,
            particles_count,
            particle_lifetime,
            particles_size: wgpu::Extent3d { width: 0, height: 0, depth_or_array_layers: 1 },
            particles_threadgroup: (0, 0, 1),
            particles_buf: None,
            particles_uniform: None,
            particles_uniform_data: ParticleUniform {
                color: [1.0; 4],
                num: [0; 2],
                point_size: 1,
                life_time: 60.0,
                fade_out_factor: 0.96,
                speed_factor: if field_type == FieldType::Field { 0.15 } else { 8.15 },
                color_ty: color_ty as i32,
                is_only_update_pos: 1,
            },
        }
    }
    pub fn update_field_type(&mut self, queue: &wgpu::Queue, ty: FieldType) -> bool {
        if self.field_type != ty {
            self.field_type = ty;
            self.particles_uniform_data.speed_factor =
                if self.field_type == FieldType::Field { 0.15 } else { 8.15 };
            self.update_particles_uniform(queue);
            true
        } else {
            false
        }
    }

    pub fn update_canvas_size(
        &mut self, device: &wgpu::Device, queue: &wgpu::Queue, canvas_size: Size<u32>,
    ) {
        self.canvas_size = canvas_size;
        self.update_particles_data(device, queue);
    }

    pub fn update_particles_count(
        &mut self, device: &wgpu::Device, queue: &wgpu::Queue, count: i32,
    ) {
        self.particles_count = count;
        self.update_particles_data(device, queue);
    }

    pub fn update_particle_color(
        &mut self, _device: &wgpu::Device, queue: &wgpu::Queue,
        color_type: crate::ParticleColorType,
    ) {
        if self.particles_uniform_data.color_ty == color_type as i32 {
            return;
        }
        self.particles_uniform_data.color_ty = color_type as i32;
        self.update_particles_uniform(queue);
    }

    pub fn update_particle_point_size(&mut self, queue: &wgpu::Queue, point_size: i32) {
        if self.particles_uniform_data.point_size == point_size {
            return;
        }
        self.particles_uniform_data.point_size = point_size;
        self.update_particles_uniform(queue);
    }

    pub fn update_particles_uniform(&self, queue: &wgpu::Queue) {
        queue.write_buffer(
            &self.particles_uniform.as_ref().unwrap().buffer,
            0,
            self.particles_uniform_data.as_bytes(),
        );
    }

    fn update_particles_data(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let (particles_size, particles_threadgroup, particles) =
            get_particles_data(self.canvas_size, self.particles_count, self.particle_lifetime);
        self.particles_size = particles_size;
        self.particles_threadgroup = particles_threadgroup;
        self.particles_uniform_data.num =
            [self.particles_size.width as i32, self.particles_size.height as i32];
        if let Some(buf) = self.particles_buf.as_ref() {
            self.update_particles_uniform(queue);
            queue.write_buffer(&buf.buffer, 0, particles.as_bytes());
        } else {
            self.particles_buf = Some(BufferObj::create_buffer(
                device,
                Some(&particles),
                None,
                wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::VERTEX,
                Some("particles_buf"),
            ));
            self.particles_uniform = Some(BufferObj::create_uniform_buffer(
                device,
                &self.particles_uniform_data,
                Some("particle_uniform"),
            ));
        }
    }
}
