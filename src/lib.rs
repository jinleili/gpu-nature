use zerocopy::{AsBytes, FromBytes};

use std::usize;

mod diffraction;
use diffraction::Diffraction;
mod canvas;
pub use canvas::Canvas;

mod noise;

use idroid::vertex::PosOnly;
use idroid::vertex::PosColor as PosTangent;

#[repr(C)]
#[derive(Copy, Clone, Debug, AsBytes, FromBytes)]
pub struct MVPMatUniform {
    mv: [[f32; 4]; 4],
    proj: [[f32; 4]; 4],
    mvp: [[f32; 4]; 4],
    normal: [[f32; 4]; 4],
}

pub fn generate_circle_plane(r: f32, fan_segment: usize) -> (Vec<PosOnly>, Vec<u32>) {
    // WebGPU 1.0 not support Triangle_Fan primitive
    let mut vertex_list: Vec<PosOnly> = Vec::with_capacity(fan_segment + 2);
    let z = -0.45_f32;
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
pub fn generate_disc_plane(min_r: f32, max_r: f32, fan_segment: usize) -> (Vec<PosTangent>, Vec<u32>) {
    // WebGPU 1.0 not support Triangle_Fan primitive
    let mut vertex_list: Vec<PosTangent> = Vec::with_capacity(fan_segment);
    let z = -0.45_f32;
    vertex_list.push(PosTangent::new([min_r, 0.0, z], [0.0, 1.0, z, 1.0]));
    vertex_list.push(PosTangent::new([max_r, 0.0, z], [0.0, 1.0, z, 1.0]));

    let tangent_r = 1.0;
    let tan_offset_angle = std::f32::consts::FRAC_PI_2;

    let mut index_list: Vec<u32> = Vec::with_capacity(fan_segment * 6);

    let step = (std::f32::consts::PI * 2.0) / fan_segment as f32;
    for i in 1..fan_segment {
        let angle = step * i as f32;
        // 切线只表达大小与方向，可以任意平移，so, Z 与平面的 Z 坐标无关
        let tangent = [tangent_r * (angle + tan_offset_angle).cos(), tangent_r * (angle + tan_offset_angle).sin(), 0.0, 1.0];
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
