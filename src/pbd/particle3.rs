use super::{particle::ParticleBufferObj, BendingConstraintObj, MeshColoringObj, StretchConstraintObj};
use idroid::math::Point3D;
use zerocopy::{AsBytes, FromBytes};

// 粒子对象

#[repr(C)]
#[derive(AsBytes, FromBytes)]
pub struct ParticleConstraints {
    // 拉伸约束 stretches 的索引
    pub stretches: [i32; 4],
    pub bendings: [i32; 4],
}

impl ParticleConstraints {
    fn new() -> Self {
        Self { stretches: [-1; 4], bendings: [-1; 4] }
    }
}

pub fn generate_cloth_particles3(
    horizontal_num: usize, vertical_num: usize, horizontal_pixel: f32, vertical_pixel: f32, a_pixel_on_ndc: f32,
) -> (
    (f32, f32, f32),
    Vec<ParticleBufferObj>,
    Vec<ParticleConstraints>,
    Vec<StretchConstraintObj>,
    Vec<BendingConstraintObj>,
    Vec<MeshColoringObj>,
    Vec<i32>,
) {
    let mut particles = Vec::with_capacity(horizontal_num * vertical_num);
    let mut particle_constraints: Vec<ParticleConstraints> = Vec::with_capacity(horizontal_num * vertical_num);
    let mut stretch_constraints: Vec<StretchConstraintObj> = Vec::with_capacity(horizontal_num * vertical_num * 4);
    let mut bending_constraints: Vec<BendingConstraintObj> = Vec::with_capacity(horizontal_num * vertical_num * 4);

    let horizontal_step = horizontal_pixel / (horizontal_num - 1) as f32 * a_pixel_on_ndc;
    let vertical_step = vertical_pixel / (vertical_num - 1) as f32 * a_pixel_on_ndc;
    let uv_x_step = 1.0 / (horizontal_num - 1) as f32;
    let uv_y_step = 1.0 / (vertical_num - 1) as f32;
    println!("step: {}, {}", horizontal_step, vertical_step);

    let tl_x = (-horizontal_step) * ((horizontal_num - 1) as f32 / 2.0);
    let tl_y = vertical_step * ((vertical_num - 1) as f32 / 2.0);
    // 图元平均边长
    let average_edge =
        (horizontal_step + vertical_step + (horizontal_step * horizontal_step + vertical_step * vertical_step).sqrt())
            / 3.0;
    let mut invert_mass = 0.1;
    for h in 0..vertical_num {
        for w in 0..horizontal_num {
            let mut p = [tl_x + horizontal_step * w as f32, tl_y - vertical_step * h as f32, 0.0, 0.0];
            // 上边两个角固定：粒子质量为 无穷大
            // 每个顶点的质量等于与之相连的每个三角形质量的 1/3 之后
            // if (h == 0 && w == 0) || (h == 0 && w == horizontal_num - 1) {
            // 上边整个固定，避免上边出现布料边缘的垂下效果
            // if h <= (vertical_num - 2) {
            if h <= 1 {
                invert_mass = 0.0;
            } else if w == 0 || w == (horizontal_num - 1) || h == (vertical_num - 1) {
                // 边界上的点，只有两个三角形与之相连
                invert_mass = 0.2;
            } else {
                invert_mass = 0.1;
            }
            // let gravity = if h > vertical_num - 4 { -2.98 } else { -1.0 - 1.98 / (vertical_num - (h + 1)) as f32 };
            let gravity = -2.98;
            particles.push(ParticleBufferObj {
                pos: p,
                old_pos: p,
                // 重力加速度不能太小，会导致布料飘来飘去，没有重量感
                accelerate: [0.0, gravity, 0.0, 0.0],
                // webgpu 的纹理坐标是左上角为 0，0
                uv_mass: [uv_x_step * w as f32, uv_y_step * h as f32, invert_mass, 0.0],
                connect: [0; 4],
            });
            // print!("({:?}, {}),", p[0], p[1]);
        }
    }
    // 与粒子直接相邻的其它粒子
    cal_connected_particles(&mut particles, horizontal_num, vertical_num);
    for h in 0..vertical_num {
        for w in 0..horizontal_num {
            let index0 = h * horizontal_num + w;
            let particle0 = &mut particles[index0];
            let mut constraints_obj = ParticleConstraints::new();
            // 将每个粒子对应的全部约束的索引装进单独的数组里
            let p0: Point3D = Point3D::new(particle0.pos[0], particle0.pos[1], particle0.pos[2]);
            if w < horizontal_num - 1 {
                constraints_obj.stretches[0] = stretch_constraints.len() as i32;
                stretch_constraints.push(get_constraint(&particles, &p0, index0, w + 1 + h * horizontal_num));
            }
            if h < vertical_num - 1 {
                constraints_obj.stretches[1] = stretch_constraints.len() as i32;
                stretch_constraints.push(get_constraint(&particles, &p0, index0, w + (h + 1) * horizontal_num));
            }
            // shear constraint
            if w < horizontal_num - 1 && h < vertical_num - 1 {
                constraints_obj.stretches[2] = stretch_constraints.len() as i32;
                stretch_constraints.push(get_constraint(&particles, &p0, index0, w + 1 + (h + 1) * horizontal_num));
            }
            // 剪切需要有十字交叉的设置, 避免运动过程中的剪切变形
            if w > 0 && h < vertical_num - 1 {
                constraints_obj.stretches[3] = stretch_constraints.len() as i32;
                stretch_constraints.push(get_constraint(&particles, &p0, index0, w - 1 + (h + 1) * horizontal_num));
            }

            // 弯曲约束
            let v = index0 as i32;
            let h0 = 0.0;
            if w > 0 && w < (horizontal_num - 1) {
                // 水平约束
                let b0 = v - 1;
                let b1 = v + 1;
                constraints_obj.bendings[0] = bending_constraints.len() as i32;
                bending_constraints.push(BendingConstraintObj { v, b0, b1, h0 });
            }
            if h > 0 && h < (vertical_num - 1) {
                // 垂直约束
                let b0 = v - horizontal_num as i32;
                let b1 = v + horizontal_num as i32;
                constraints_obj.bendings[1] = bending_constraints.len() as i32;
                bending_constraints.push(BendingConstraintObj { v, b0, b1, h0 });
            }
            if w > 0 && w < (horizontal_num - 1) && h > 0 && h < (vertical_num - 1) {
                // 斜向约束
                let b0 = v + 1 - horizontal_num as i32;
                let b1 = v - 1 + horizontal_num as i32;
                constraints_obj.bendings[2] = bending_constraints.len() as i32;
                bending_constraints.push(BendingConstraintObj { v, b0, b1, h0 });
            }
            particle_constraints.push(constraints_obj);
        }
    }
    // 将默认位置折叠在最顶边
    // let mut py = tl_y;
    // let mut pz = 0.0_f32;
    // let mut step_z = 0.005;
    // let max_z = 0.024;
    // for h in 0..vertical_num {
    //     py = tl_y - (h as f32 * vertical_step) * 0.25;
    //     pz += step_z;
    //     if pz > max_z || pz < (-max_z) {
    //         step_z *= -1.0;
    //     }
    //     for w in 0..horizontal_num {
    //         let index = h * horizontal_num + w;
    //         let mut pos = particles[index].pos;
    //         pos[1] = py;
    //         pos[2] = pz;
    //         particles[index].pos = pos;
    //         particles[index].old_pos = pos;
    //     }
    // }

    // 约束归组新相法：
    // 先安 gap = 3 直接分组
    // 然后找出约束最少一组重新分配：只要p0 在那个组里没有重复，就可将其归入那组
    let mut mesh_colorings: Vec<MeshColoringObj> = vec![];
    let mut offset = 0;
    let step: u32 = 8;
    let mut mesh_coloring_buf: Vec<i32> = vec![];
    for h_start in 0..3 {
        for w_start in 0..3 {
            let mut h = h_start;
            let mut group_len = 0;
            while h < vertical_num {
                let mut w = w_start;
                while w < horizontal_num {
                    let index = h * horizontal_num + w;
                    mesh_coloring_buf.push(index as i32);
                    group_len += 1;
                    w += 3;
                }
                h += 3;
            }
            let mut max_num_x: u32 = 0;
            let mut max_num_y: u32 = 0;
            for i in 1..100 {
                max_num_x = i * step;
                if max_num_x * max_num_y >= group_len {
                    max_num_x = cal_real_max_num(max_num_x, max_num_y, group_len);
                    break;
                }
                max_num_y = i * step;
                if max_num_x * max_num_y >= group_len {
                    max_num_y = cal_real_max_num(max_num_y, max_num_x, group_len);
                    break;
                }
            }
            mesh_colorings.push(MeshColoringObj {
                offset,
                max_num_x,
                max_num_y,
                group_len,
                thread_group: ((max_num_x + 7) / step, (max_num_y + 7) / step),
            });
            offset += group_len;
            println!("new group: {}", group_len);
        }
    }
    println!("新分组法：{}", mesh_coloring_buf.len());
    // println!("{:?}", particle_constraints);
    (
        (tl_x, tl_y, average_edge),
        particles,
        particle_constraints,
        stretch_constraints,
        bending_constraints,
        mesh_colorings,
        mesh_coloring_buf,
    )
}

fn cal_connected_particles(particles: &mut Vec<ParticleBufferObj>, horizontal_num: usize, vertical_num: usize) {
    for h in 0..vertical_num {
        for w in 0..horizontal_num {
            let index0 = h * horizontal_num + w;
            let particle0 = &mut particles[index0];

            if h == 0 {
                if w == 0 {
                    // 左上角
                    particle0.connect[0] = index0 as i32 + 1;
                    particle0.connect[1] = (index0 + horizontal_num) as i32;
                    particle0.connect[2] = particle0.connect[0];
                    particle0.connect[3] = particle0.connect[1];
                } else if w == horizontal_num - 1 {
                    // 右上角
                    particle0.connect[0] = (index0 + horizontal_num) as i32;
                    particle0.connect[1] = (index0 - 1) as i32;
                    particle0.connect[2] = particle0.connect[0];
                    particle0.connect[3] = particle0.connect[1];
                } else {
                    particle0.connect[0] = index0 as i32 + 1;
                    particle0.connect[1] = (index0 + horizontal_num) as i32;
                    particle0.connect[2] = particle0.connect[1];
                    particle0.connect[3] = (index0 - 1) as i32;
                }
            } else if h == vertical_num - 1 {
                if w == 0 {
                    // 左下角
                    particle0.connect[0] = (index0 - horizontal_num) as i32;
                    particle0.connect[1] = (index0 + 1) as i32;
                    particle0.connect[2] = particle0.connect[0];
                    particle0.connect[3] = particle0.connect[1];
                } else if w == horizontal_num - 1 {
                    // 右下角
                    particle0.connect[0] = (index0 - 1) as i32;
                    particle0.connect[1] = (index0 - horizontal_num) as i32;
                    particle0.connect[2] = particle0.connect[0];
                    particle0.connect[3] = particle0.connect[1];
                } else {
                    // 底边
                    particle0.connect[0] = (index0 - 1) as i32;
                    particle0.connect[1] = (index0 - horizontal_num) as i32;
                    particle0.connect[2] = particle0.connect[1];
                    particle0.connect[3] = (index0 + 1) as i32;
                }
            } else {
                if w == 0 {
                    // 左竖边
                    particle0.connect[0] = (index0 - horizontal_num) as i32;
                    particle0.connect[1] = (index0 + 1) as i32;
                    particle0.connect[2] = particle0.connect[1];
                    particle0.connect[3] = (index0 + horizontal_num) as i32;
                } else if w == horizontal_num - 1 {
                    // 右竖边
                    particle0.connect[0] = (index0 + horizontal_num) as i32;
                    particle0.connect[1] = (index0 - 1) as i32;
                    particle0.connect[2] = particle0.connect[1];
                    particle0.connect[3] = (index0 - horizontal_num) as i32;
                } else {
                    particle0.connect[0] = (index0 - horizontal_num) as i32;
                    particle0.connect[1] = (index0 + 1) as i32;
                    particle0.connect[2] = (index0 + horizontal_num) as i32;
                    particle0.connect[3] = (index0 - 1) as i32;
                }
            }
        }
    }
}

fn get_constraint(
    particles: &Vec<ParticleBufferObj>, p0: &Point3D, index0: usize, index1: usize,
) -> StretchConstraintObj {
    let particle1 = &particles[index1];
    let p1: Point3D = Point3D::new(particle1.pos[0], particle1.pos[1], particle1.pos[2]);
    let rest_length: f32 = p0.minus(&p1).length();
    StretchConstraintObj { rest_length, lambda: 0.0, particle1: index1 as i32, particle0: index0 as i32 }
}

// 计算 thread_group 维度
fn cal_real_max_num(max_num: u32, other: u32, group_len: u32) -> u32 {
    for i in 0..8 {
        let val = max_num - i;
        if val * other == group_len {
            return val;
        } else if val * other < group_len {
            return max_num - i + 1;
        }
    }
    return max_num - 7;
}
