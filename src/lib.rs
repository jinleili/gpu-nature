use zerocopy::{AsBytes, FromBytes};

use app_surface::math::{Position, Size};
use std::usize;

mod field_player;
use field_player::FieldPlayer;
mod field_velocity_code;
use field_velocity_code::get_velocity_code_segment;

mod setting_obj;
pub use setting_obj::SettingObj;

mod fluid;
use fluid::{D3FluidPlayer, FluidPlayer};

mod combinate_canvas;
pub use combinate_canvas::CombinateCanvas;

mod diffraction;
use diffraction::Diffraction;
mod canvas;
pub use canvas::Canvas;

mod floor;
use floor::Floor;

mod noise;

mod pbd;
pub use pbd::PBDCanvas;

mod brick;
pub use brick::Brick;

#[cfg(target_os = "ios")]
mod ffi_ios;
#[cfg(target_os = "ios")]
pub use ffi_ios::*;

#[cfg(target_arch = "wasm32")]
mod web;
#[cfg(target_arch = "wasm32")]
pub use web::*;

#[cfg(not(target_arch = "wasm32"))]
pub use std::println as console_log;

mod util;
#[cfg(not(target_arch = "wasm32"))]
use util::shader::{create_shader_module, insert_code_then_create};
#[cfg(target_arch = "wasm32")]
use web::{create_shader_module, insert_code_then_create};

use util::vertex::PosColor as PosTangent;
use util::vertex::PosOnly;

#[repr(C)]
#[derive(Copy, Clone, Debug, AsBytes, FromBytes)]
pub struct MVPMatUniform {
    mv: [[f32; 4]; 4],
    proj: [[f32; 4]; 4],
    mvp: [[f32; 4]; 4],
    normal: [[f32; 4]; 4],
}

trait Player {
    fn update_uniforms(&mut self, _queue: &wgpu::Queue, _setting: &crate::SettingObj) {}

    fn on_click(&mut self, _device: &wgpu::Device, _queue: &wgpu::Queue, _pos: Position) {}

    fn touch_begin(&mut self, _device: &wgpu::Device, _queue: &wgpu::Queue) {}

    fn touch_move(&mut self, _device: &wgpu::Device, _queue: &wgpu::Queue, _pos: Position) {}

    fn touch_end(&mut self, _device: &wgpu::Device, _queue: &wgpu::Queue) {}

    fn reset(&mut self, _device: &wgpu::Device, _queue: &wgpu::Queue) {}

    fn enter_frame(
        &mut self, device: &wgpu::Device, queue: &wgpu::Queue, frame_view: &wgpu::TextureView,
        _setting: &mut crate::SettingObj,
    );
}

#[derive(Clone, Copy, PartialEq)]
pub enum FieldType {
    Field,
    Fluid,
    D3Fluid,
}

#[derive(Clone, Copy, PartialEq)]
pub enum FieldAnimationType {
    Basic,
    JuliaSet,
    Spirl,
    Poiseuille,
    LidDrivenCavity,
    Custom,
}

#[derive(Clone, Copy)]
pub enum ParticleColorType {
    Uniform = 0,
    MovementAngle = 1,
    Speed = 2,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, AsBytes, FromBytes)]
pub struct FieldUniform {
    // field lattice number
    pub lattice_size: [i32; 4],
    pub lattice_pixel_size: [f32; 4],
    // canvas pixel number
    pub canvas_size: [i32; 4],
    pub normalized_space_size: [f32; 4],
    pub pixel_distance: [f32; 4],
    // 0: pixel speed, field player used
    // 1: lbm lattice speed, fluid player used. Its value is usually no greater than 0.2
    pub speed_ty: i32,
    // align to 16 * n
    pub _padding: [f32; 3],
}
#[repr(C)]
#[derive(Copy, Clone, AsBytes, FromBytes)]
pub struct ParticleUniform {
    // particle uniform color
    pub color: [f32; 4],
    // total particle number
    pub num: [i32; 2],
    pub point_size: i32,
    pub life_time: f32,
    pub fade_out_factor: f32,
    pub speed_factor: f32,
    // particle color type
    // 0: uniform color; 1: use velocity as particle color, 2: angle as color
    pub color_ty: i32,
    // 1: not draw on the canvas
    pub is_only_update_pos: i32,
}

#[repr(C)]
#[derive(Copy, Clone, AsBytes, FromBytes)]
pub struct TrajectoryParticle {
    pub pos: [f32; 2],
    pub pos_initial: [f32; 2],
    pub life_time: f32,
    pub fade: f32,
}

impl TrajectoryParticle {
    pub fn zero() -> Self {
        TrajectoryParticle { pos: [0.0, 0.0], pos_initial: [0.0, 0.0], life_time: 0.0, fade: 0.0 }
    }
}

#[repr(C)]
#[derive(AsBytes, FromBytes)]
pub struct Particle3D {
    pub pos: [f32; 4],
    // initial position, use to reset particle position
    pos_initial: [f32; 4],
}

#[repr(C)]
#[derive(Copy, Clone, AsBytes, FromBytes)]
struct TrajectoryUniform {
    screen_factor: [f32; 2],
    // which view particles position will drawing to.
    trajectory_view_index: i32,
    bg_view_index: i32,
}

#[repr(C)]
#[derive(Copy, Clone, AsBytes, FromBytes)]
pub struct Pixel {
    pub alpha: f32,
    // absolute velocity
    pub speed: f32,
    // density
    pub rho: f32,
}

use rand::{prelude::Distribution, Rng};

const MAX_PARTICLE_COUNT: usize = 205000;
fn get_particles_data(
    canvas_size: Size<u32>, count: i32, life_time: f32,
) -> (wgpu::Extent3d, (u32, u32, u32), Vec<TrajectoryParticle>) {
    let x = (count as f32 * (canvas_size.width as f32 / canvas_size.height as f32)).sqrt().ceil();
    let particles_size = wgpu::Extent3d {
        width: x as u32,
        height: (x * (canvas_size.height as f32 / canvas_size.width as f32)).ceil() as u32,
        depth_or_array_layers: 1,
    };
    let threadgroup = ((particles_size.width + 15) / 16, (particles_size.height + 15) / 16, 1);

    let mut particles = init_trajectory_particles(canvas_size, particles_size, life_time);
    if MAX_PARTICLE_COUNT > particles.len() {
        for _i in 0..(MAX_PARTICLE_COUNT - particles.len()) {
            particles.push(TrajectoryParticle::zero());
        }
    }
    (particles_size, threadgroup, particles)
}

fn init_trajectory_particles(
    canvas_size: Size<u32>, num: wgpu::Extent3d, life_time: f32,
) -> Vec<TrajectoryParticle> {
    let mut data: Vec<TrajectoryParticle> = vec![];
    let mut rng = rand::thread_rng();
    let step_x = canvas_size.width as f32 / (num.width - 1) as f32;
    let step_y = canvas_size.height as f32 / (num.height - 1) as f32;
    let unif_x = rand::distributions::Uniform::new_inclusive(-step_x, step_x);
    let unif_y = rand::distributions::Uniform::new_inclusive(-step_y, step_y);
    let unif_life = rand::distributions::Uniform::new_inclusive(
        0.0,
        if life_time <= 0.0 { 1.0 } else { life_time },
    );

    for x in 0..num.width {
        let pixel_x = step_x * x as f32;
        for y in 0..num.height {
            let pos =
                [pixel_x + unif_x.sample(&mut rng), step_y * y as f32 + unif_y.sample(&mut rng)];
            let pos_initial =
                if life_time <= 1.0 { [rng.gen_range(0.0, step_x), pos[1]] } else { pos };
            data.push(TrajectoryParticle {
                pos,
                pos_initial,
                life_time: if life_time <= 1.0 { 0.0 } else { unif_life.sample(&mut rng) },
                fade: 0.0,
            });
        }
    }

    data
}

fn init_3d_particles(num: wgpu::Extent3d) -> Vec<Particle3D> {
    let mut data: Vec<Particle3D> = vec![];
    let mut rng = rand::thread_rng();
    let step_x = 2.0 / num.width as f32;
    let step_y = 2.0 / num.height as f32;
    let step_z = 2.0 / num.depth_or_array_layers as f32;

    let unif_x = rand::distributions::Uniform::new_inclusive(-step_x, step_x);
    let unif_y = rand::distributions::Uniform::new_inclusive(-step_y, step_y);
    let unif_z = rand::distributions::Uniform::new_inclusive(-step_z, step_z);

    for x in 0..num.width {
        let pixel_x = step_x * x as f32;
        for y in 0..num.height {
            for z in 0..num.depth_or_array_layers {
                let pos = [
                    -1.0 + pixel_x + unif_x.sample(&mut rng),
                    -1.0 + step_y * y as f32 + unif_y.sample(&mut rng),
                    -1.0 + step_z * z as f32 + unif_z.sample(&mut rng),
                    1.0,
                ];
                let pos_initial = [rng.gen_range(0.0, step_x), pos[1], pos[2], 1.0];
                data.push(Particle3D { pos, pos_initial });
            }
        }
    }

    data
}

pub fn generate_circle_plane(r: f32, fan_segment: usize) -> (Vec<PosOnly>, Vec<u32>) {
    // WebGPU 1.0 not support Triangle_Fan primitive
    let mut vertex_list: Vec<PosOnly> = Vec::with_capacity(fan_segment + 2);
    let z = 0.0_f32;
    vertex_list.push(PosOnly::new([0.0, 0.0, z]));
    vertex_list.push(PosOnly::new([r, 0.0, z]));

    let mut index_list: Vec<u32> = Vec::with_capacity(fan_segment * 3);

    let step = (std::f32::consts::PI * 2.0) / fan_segment as f32;
    for i in 1..=fan_segment {
        let angle = step * i as f32;
        vertex_list.push(PosOnly::new([r * angle.cos(), r * angle.sin(), z]));
        index_list.push(0);
        index_list.push(i as u32);
        if i == fan_segment {
            index_list.push(1);
        } else {
            index_list.push(i as u32 + 1);
        }
    }
    return (vertex_list, index_list);
}

// 光盘平面
pub fn generate_disc_plane(
    min_r: f32, max_r: f32, fan_segment: usize,
) -> (Vec<PosTangent>, Vec<u32>) {
    // WebGPU 1.0 not support Triangle_Fan primitive
    let mut vertex_list: Vec<PosTangent> = Vec::with_capacity(fan_segment);
    let z = 0.0_f32;
    vertex_list.push(PosTangent::new([min_r, 0.0, z], [0.0, 1.0, z, 1.0]));
    vertex_list.push(PosTangent::new([max_r, 0.0, z], [0.0, 1.0, z, 1.0]));

    let tangent_r = 1.0;
    let tan_offset_angle = std::f32::consts::FRAC_PI_2;

    let mut index_list: Vec<u32> = Vec::with_capacity(fan_segment * 6);

    let step = (std::f32::consts::PI * 2.0) / fan_segment as f32;
    for i in 1..fan_segment {
        let angle = step * i as f32;
        // 切线只表达大小与方向，可以任意平移，so, Z 与平面的 Z 坐标无关
        let tangent = [
            tangent_r * (angle + tan_offset_angle).cos(),
            tangent_r * (angle + tan_offset_angle).sin(),
            0.0,
            1.0,
        ];
        vertex_list.push(PosTangent::new([min_r * angle.cos(), min_r * angle.sin(), z], tangent));
        vertex_list.push(PosTangent::new([max_r * angle.cos(), max_r * angle.sin(), z], tangent));
        let index = i as u32 * 2;
        index_list.append(&mut vec![index - 2, index - 1, index, index, index - 1, index + 1]);
    }
    let index = (fan_segment - 1) as u32 * 2;
    index_list.append(&mut vec![index, index + 1, 0, 0, index + 1, 1]);
    // println!("{:?}", index_list);

    return (vertex_list, index_list);
}
