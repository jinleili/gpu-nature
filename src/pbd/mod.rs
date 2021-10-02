mod particle;
pub use particle::generate_cloth_particles;
mod particle3;
pub use particle3::generate_cloth_particles3;

mod cloth;
pub use cloth::Cloth;
mod cloth3;
pub use cloth3::Cloth3;

mod pbd_canvas;
pub use pbd_canvas::PBDCanvas;

mod line_particle;
pub use line_particle::generate_line_particles;
mod line;
pub use line::Line;

use zerocopy::{AsBytes, FromBytes};

#[repr(C)]
#[derive(Copy, Clone, AsBytes, FromBytes)]
struct TriangleObj {
    p0: i32,
    p1: i32,
    p2: i32,
}

#[repr(C)]
#[derive(Copy, Clone, AsBytes, FromBytes)]
pub struct FrameUniform {
    // 帧绘制计数
    frame_index: i32,
}

#[repr(C)]
#[derive(Copy, Clone, AsBytes, FromBytes)]
pub struct ClothUniform {
    // 粒子个数
    num_x: i32,
    num_y: i32,
    triangle_num: i32,
    compliance: f32,
    dt: f32,
}

#[repr(C)]
#[derive(Copy, Clone, AsBytes, FromBytes)]
pub struct BinUniform {
    // bin hash 容器数
    bin_num: [i32; 4],
    // 容器各轴向上最大的索引数
    bin_max_index: [i32; 4],
    bin_size: [f32; 4],
    // 转换到 【0～n]坐标空间需要的偏移
    pos_offset: [f32; 4],
    max_bin_count: i32,
    padding: [f32; 3],
}

// 拉伸 | 距离约束
#[repr(C)]
#[derive(Copy, Clone, AsBytes, FromBytes)]
pub struct StretchConstraintObj {
    pub rest_length: f32,
    pub lambda: f32,
    pub particle0: i32,
    pub particle1: i32,
}
// 弯曲约束
#[repr(C)]
#[derive(Copy, Clone, AsBytes, FromBytes)]
pub struct BendingConstraintObj {
    pub v: i32,
    pub b0: i32,
    pub b1: i32,
    pub h0: f32,
}

#[repr(C)]
#[derive(Copy, Clone, AsBytes, FromBytes, Debug)]
pub struct BendingPushConstants {
    offset: i32,
    max_num_x: i32,
    // 当前 mesh coloring 分组的数据长度
    group_len: i32,
    // 迭代計數的倒數
    invert_iter: f32,
}

// 约束的网络着色分组
#[derive(Debug)]
pub struct MeshColoringObj {
    pub offset: u32,
    pub max_num_x: u32,
    pub max_num_y: u32,
    pub group_len: u32,
    pub thread_group: (u32, u32),
}

impl MeshColoringObj {
    pub fn get_push_constants_data(&self) -> Vec<u32> {
        vec![self.offset, self.max_num_x, self.max_num_y, self.group_len]
    }

    pub fn get_bending_push_constants_data(&self, iter_count: i32) -> BendingPushConstants {
        BendingPushConstants {
            offset: self.offset as i32,
            max_num_x: self.max_num_x as i32,
            group_len: self.group_len as i32,
            invert_iter: 1.0 / (iter_count + 1) as f32,
        }
    }
}
