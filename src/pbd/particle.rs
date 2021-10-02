use super::MeshColoringObj;
use idroid::math::Point3D;
use zerocopy::{AsBytes, FromBytes};

// 粒子对象
#[repr(C)]
#[derive(Copy, Clone, Debug, AsBytes, FromBytes)]
pub struct ParticleBufferObj {
    pub pos: [f32; 4],
    pub old_pos: [f32; 4],
    pub accelerate: [f32; 4],
    // uv and invert_mass
    // 为了字节对齐
    pub uv_mass: [f32; 4],
    // 与之相连的4个粒子的索引，用于计算法线
    pub connect: [i32; 4],
}
// 约束对象
#[repr(C)]
#[derive(Copy, Clone, AsBytes, FromBytes)]
pub struct ConstraintBufferObj {
    pub rest_length: f32,
    pub lambda: f32,
    pub particle0: i32,
    pub particle1: i32,
}

impl ConstraintBufferObj {
    pub fn is_contain_particles(&self, other: &ConstraintBufferObj) -> bool {
        if self.particle0 == other.particle0
            || self.particle0 == other.particle1
            || self.particle1 == other.particle0
            || self.particle1 == other.particle1
        {
            // print!("({}, {}, {}, {}) ;;", self.particle0, self.particle1, other.particle0, other.particle1);
            true
        } else {
            false
        }
    }
}

// 约束对象
#[repr(C)]
#[derive(Copy, Clone, AsBytes, FromBytes)]
pub struct BendConstraintBufferObj {
    pub p0: i32,
    pub p1: i32,
    pub p2: i32,
    pub p3: i32,
}

impl BendConstraintBufferObj {
    pub fn is_contain(&self, other: &BendConstraintBufferObj) -> bool {
        let list0 = [self.p0, self.p1, self.p1, self.p3];
        let list1 = [other.p0, other.p1, other.p1, other.p3];

        for i in list0.iter() {
            for j in list1.iter() {
                if i.clone() == j.clone() {
                    return true;
                }
            }
        }
        false
    }
}

pub fn generate_cloth_particles(
    horizontal_num: usize, vertical_num: usize, horizontal_pixel: f32, vertical_pixel: f32,
    a_pixel_on_ndc: f32,
) -> (
    (f32, f32),
    Vec<ParticleBufferObj>,
    Vec<ConstraintBufferObj>,
    Vec<MeshColoringObj>,
    Vec<[i32; 3]>,
    Vec<[i32; 3]>,
    Vec<MeshColoringObj>,
    Vec<BendConstraintBufferObj>,
    Vec<[i32; 3]>,
) {
    let mut particles = Vec::with_capacity(horizontal_num * vertical_num);
    let mut constraints = Vec::with_capacity(horizontal_num * vertical_num * 3);
    let mut stretch_constraints: Vec<[i32; 3]> = Vec::with_capacity(horizontal_num * vertical_num);

    let horizontal_step = horizontal_pixel / (horizontal_num - 1) as f32 * a_pixel_on_ndc;
    let vertical_step = vertical_pixel / (vertical_num - 1) as f32 * a_pixel_on_ndc;
    let uv_x_step = 1.0 / (horizontal_num - 1) as f32;
    let uv_y_step = 1.0 / (vertical_num - 1) as f32;

    let tl_x = (-horizontal_step) * ((horizontal_num - 1) as f32 / 2.0);
    let tl_y = vertical_step * ((vertical_num - 1) as f32 / 2.0);
    let mut invert_mass = 0.1;
    for h in 0..vertical_num {
        for w in 0..horizontal_num {
            let mut p =
                [tl_x + horizontal_step * w as f32, tl_y - vertical_step * h as f32, 0.0, 0.0];
            // 上边两个角固定：粒子质量为 无穷大
            // 每个顶点的质量等于与之相连的每个三角形质量的 1/3 之后
            // if (h == 0 && w == 0) || (h == 0 && w == horizontal_num - 1) {
            // 上边整个固定，避免上边出现布料边缘的垂下效果
            if h == 0 {
                invert_mass = 0.0;
            } else if w == 0 || w == (horizontal_num - 1) || h == (vertical_num - 1) {
                // 边界上的点，只有两个三角形与之相连
                invert_mass = 0.2;
            } else {
                invert_mass = 0.1;
            }
            particles.push(ParticleBufferObj {
                pos: p,
                old_pos: p,
                // 重力加速度不能太小，会导致布料飘来飘去，没有重量感
                accelerate: [0.0, -3.98, 0.0, 0.0],
                // webgpu 的纹理坐标是左上角为 0，0
                uv_mass: [uv_x_step * w as f32, uv_y_step * h as f32, invert_mass, 0.0],
                connect: [0; 4],
            });
        }
    }
    // 与粒子直接相邻的其它粒子
    cal_connected_particles(&mut particles, horizontal_num, vertical_num);
    for h in 0..vertical_num {
        for w in 0..horizontal_num {
            let index0 = h * horizontal_num + w;
            let particle0 = &mut particles[index0];

            // 将每个粒子对应的全部约束的索引装进单独的数组里
            let mut group: [i32; 3] = [-1; 3];
            let p0: Point3D = Point3D::new(particle0.pos[0], particle0.pos[1], particle0.pos[2]);
            if w < horizontal_num - 1 {
                group[0] = constraints.len() as i32;
                constraints.push(get_constraint(
                    &particles,
                    &p0,
                    index0,
                    w + 1 + h * horizontal_num,
                ));
            }
            if h < vertical_num - 1 {
                group[1] = constraints.len() as i32;
                constraints.push(get_constraint(
                    &particles,
                    &p0,
                    index0,
                    w + (h + 1) * horizontal_num,
                ));
            }
            // shear constraint
            if w < horizontal_num - 1 && h < vertical_num - 1 {
                group[2] = constraints.len() as i32;
                constraints.push(get_constraint(
                    &particles,
                    &p0,
                    index0,
                    w + 1 + (h + 1) * horizontal_num,
                ));
            }
            stretch_constraints.push(group);
        }
    }
    // 将默认位置折叠在最顶边
    let mut py = tl_y;
    let mut pz = 0.0_f32;
    let mut step_z = 0.005;
    let max_z = 0.029;
    let mut step_y = -vertical_step;
    let max_y = tl_y - 0.1;

    // for h in 0..vertical_num {
    //     py += h as f32 * step_y;
    //     pz += step_z;
    //     if py < max_y || py > (tl_y - vertical_step) {
    //         step_y *= -1.0;
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
    // for particle in particles.iter_mut() {
    //     particle.pos[1] = tl_y;
    //     particle.old_pos[1] = tl_y;
    // }

    let (mesh_coloring, reorder_constraints) =
        group_distance_constraints(&constraints, &stretch_constraints);

    let (bend_mesh_coloring, bend_constraint, reorder_bendings) =
        cal_bend_constraints(horizontal_num, vertical_num);
    // println!("{:?}", particle_constraints);
    (
        (tl_x, tl_y),
        particles,
        constraints,
        mesh_coloring,
        stretch_constraints,
        reorder_constraints,
        bend_mesh_coloring,
        bend_constraint,
        reorder_bendings,
    )
}

fn cal_connected_particles(
    particles: &mut Vec<ParticleBufferObj>, horizontal_num: usize, vertical_num: usize,
) {
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
) -> ConstraintBufferObj {
    let particle1 = &particles[index1];
    let p1: Point3D = Point3D::new(particle1.pos[0], particle1.pos[1], particle1.pos[2]);
    let rest_length: f32 = p0.minus(&p1).length();
    ConstraintBufferObj {
        rest_length,
        lambda: 0.0,
        particle1: index1 as i32,
        particle0: index0 as i32,
    }
}

// 计算弯曲约束，每一个顶点有三对约束,
fn cal_bend_constraints(
    horizontal_num: usize, vertical_num: usize,
) -> (Vec<MeshColoringObj>, Vec<BendConstraintBufferObj>, Vec<[i32; 3]>) {
    // 共享同一顶点的三个三角形对
    let mut index_groups: Vec<[i32; 3]> = vec![];
    let mut bendings: Vec<BendConstraintBufferObj> =
        Vec::with_capacity(horizontal_num * vertical_num * 3);
    let mut reorder_bendings: Vec<[i32; 3]> = vec![];
    // 按没有共同顶点的约束分组，再基于此生成 reorder_bendings
    let mut bending_groups: Vec<Vec<[i32; 3]>> = vec![vec![]];

    for h in 0..vertical_num {
        for w in 0..horizontal_num {
            let p0 = (h * horizontal_num + w) as i32;
            let mut a_group: [i32; 3] = [-1; 3];
            if w > 0 && w < (horizontal_num - 1) && h < (vertical_num - 1) {
                let p1 = p0 + horizontal_num as i32;
                let p2 = p0 - 1;
                let p3 = p1 + 1;
                a_group[0] = bendings.len() as i32;
                bendings.push(BendConstraintBufferObj { p0, p1, p2, p3 });
            }
            if w < (horizontal_num - 1) && h < (vertical_num - 1) {
                let p1 = p0 + horizontal_num as i32 + 1;
                let p2 = p1 - 1;
                let p3 = p0 + 1;
                a_group[1] = bendings.len() as i32;
                bendings.push(BendConstraintBufferObj { p0, p1, p2, p3 });
                if h > 0 {
                    let p1 = p0 + 1;
                    let p2 = p1 + horizontal_num as i32;
                    let p3 = p0 - horizontal_num as i32;
                    a_group[2] = bendings.len() as i32;
                    bendings.push(BendConstraintBufferObj { p0, p1, p2, p3 });
                }
            }
            index_groups.push(a_group);
        }
    }
    // 创建分组
    'outter0: for p in index_groups.iter() {
        'outter1: for a_group in bending_groups.iter_mut() {
            'inner0: for a_p in a_group.iter() {
                'inner1: for i in p.iter() {
                    if i.clone() == -1 {
                        continue 'inner1;
                    }
                    let c = &bendings[i.clone() as usize];
                    'inner2: for j in a_p.iter() {
                        if j.clone() == -1 {
                            continue 'inner2;
                        }
                        let b = &bendings[j.clone() as usize];
                        if b.is_contain(&c) {
                            continue 'outter1;
                        }
                    }
                }
            }
            // 如果分组里没有此约束组，则加入
            a_group.push(p.clone());
            continue 'outter0;
        }
        let new_group: Vec<[i32; 3]> = vec![p.clone()];
        bending_groups.push(new_group);
    }

    let mut mesh_colorings: Vec<MeshColoringObj> = vec![];
    let mut offset = 0;
    let step: u32 = 8;
    for a_group in bending_groups.iter_mut() {
        println!("bend group len: {}", a_group.len());
        let group_len = a_group.len() as u32;
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
        reorder_bendings.append(a_group);
        offset += group_len;
    }
    (mesh_colorings, bendings, reorder_bendings)
}

fn group_distance_constraints(
    constraints: &[ConstraintBufferObj], particle_constraints: &Vec<[i32; 3]>,
) -> (Vec<MeshColoringObj>, Vec<[i32; 3]>) {
    let mut groups: Vec<Vec<[i32; 3]>> = vec![vec![]];

    'outer: for pcs in particle_constraints.iter() {
        'inner: for a_group in groups.iter_mut() {
            if is_group_contain_pcs(constraints, a_group, pcs) {
                continue 'inner;
            } else {
                a_group.push(pcs.clone());
                continue 'outer;
            }
        }
        let mut a_group: Vec<[i32; 3]> = vec![];
        a_group.push(pcs.clone());
        groups.push(a_group);
    }
    // 不同的归组方法得到的数组不一样
    // 直接从头到尾遍历，会使得分组更多, 且每组数据量差异极大（最后一组更可能只包含一个粒子的约束组）
    // 由于当前使用的约束关系最远只到邻居的邻居，按照 3 * n + 1 <= vertical_num 做为分割来分两次遍历
    // 得到的分组数落在 【9，12】区间
    // let mult = horizontal_num / 2;
    // let row = if (mult * 2 + 1) > horizontal_num { (mult - 1) * 2 + 1 } else { mult * 2 + 1 };
    // for h in 0..vertical_num {
    //     for w in 0..row {
    //         let index = h * horizontal_num + w;
    //         iter_groups(index, constraints, particle_constraints, &mut groups);
    //     }
    // }
    // for h in 0..vertical_num {
    //     for w in row..horizontal_num {
    //         let index = h * horizontal_num + w;
    //         iter_groups(index, constraints, particle_constraints, &mut groups);
    //     }
    // }

    let mut mesh_colorings: Vec<MeshColoringObj> = vec![];
    let mut reorder_constraints: Vec<[i32; 3]> = vec![];
    println!("组数： {}", groups.len());
    let mut offset = 0;
    for a_group in groups.iter_mut() {
        println!("group len: {}", a_group.len());

        let group_len = a_group.len() as u32;
        mesh_colorings.push(MeshColoringObj {
            offset,
            max_num_x: 0,
            max_num_y: 0,
            group_len,
            thread_group: (((group_len + 31) as f32 / 32.0).floor() as u32, 1),
        });
        reorder_constraints.append(a_group);
        offset += group_len;
    }
    (mesh_colorings, reorder_constraints)
}

fn iter_groups(
    index: usize, constraints: &[ConstraintBufferObj], particle_constraints: &Vec<[i32; 3]>,
    groups: &mut Vec<Vec<[i32; 3]>>,
) {
    let pcs = &particle_constraints[index];
    let mut need_new_group = true;
    for a_group in groups.iter_mut() {
        if is_group_contain_pcs(constraints, a_group, pcs) {
            continue;
        } else {
            a_group.push(pcs.clone());
            need_new_group = false;
            break;
        }
    }
    if need_new_group {
        let mut a_group: Vec<[i32; 3]> = vec![];
        a_group.push(pcs.clone());
        groups.push(a_group);
    }
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

fn is_group_contain_pcs(
    constraints: &[ConstraintBufferObj], a_group: &Vec<[i32; 3]>, pcs: &[i32],
) -> bool {
    let mut is_contain = false;
    'outer: for c in pcs.iter() {
        let c = c.clone();
        if c == -1 {
            continue;
        }
        let constraint0 = &constraints[c as usize];
        for ag in a_group.iter() {
            for a in ag.iter() {
                let a = a.clone();
                if a == -1 {
                    continue;
                }
                let constraint1 = &constraints[a as usize];
                if constraint1.is_contain_particles(constraint0) {
                    is_contain = true;
                    break 'outer;
                }
            }
        }
    }
    is_contain
}
