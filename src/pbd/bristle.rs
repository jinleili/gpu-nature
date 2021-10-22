use super::{BendingConstraintObj, MeshColoringObj};

use zerocopy::{AsBytes, FromBytes};

// 粒子对象
#[repr(C)]
#[derive(Copy, Clone, AsBytes, FromBytes)]
pub struct BristleParticle {
    // w 分量为 invert_mass
    pub pos: [f32; 4],
    pub old_pos: [f32; 4],
    // 与之相连的4个粒子的索引，用于计算法线
    pub connect: [i32; 4],
}

// 拉伸 | 距离约束
#[repr(C)]
#[derive(Copy, Clone, AsBytes, FromBytes)]
pub struct StretchConstraint {
    pub rest_length: f32,
    pub particle0: i32,
    pub particle1: i32,
}

pub fn generate_bristles(
    _stroke_width: f32,
) -> (
    Vec<BristleParticle>,
    Vec<u32>,
    Vec<StretchConstraint>,
    Vec<[i32; 3]>,
    Vec<MeshColoringObj>,
    Vec<BendingConstraintObj>,
    Vec<[i32; 2]>,
    Vec<MeshColoringObj>,
) {
    let bristle_num = 20;
    let r = 0.2;
    let step_r = vec![
        r,
        r,
        r * 0.9,
        r * 0.75,
        r * 0.6,
        r * 0.5,
        r * 0.4,
        r * 0.3,
        r * 0.22,
        r * 0.16,
        r * 0.1,
        r * 0.07,
        r * 0.04,
    ];

    let step_angle = (std::f32::consts::PI * 2.0) / (bristle_num as f32);
    let distance = vec![
        0.0,
        r,
        r * 0.8,
        r * 0.6,
        r * 0.5,
        r * 0.4,
        r * 0.3,
        r * 0.25,
        r * 0.2,
        r * 0.15,
        r * 0.125,
        r * 0.1,
    ];

    let mut particles = Vec::with_capacity(bristle_num * 10);
    let mut index_data: Vec<u32> = Vec::new();
    let mut constraints: Vec<StretchConstraint> = Vec::with_capacity(bristle_num * 20);

    let mut index = 0;
    for i in 0..bristle_num {
        let mut h = 0.0;
        // 笔尖在原点
        for d in distance.iter() {
            h -= d;
        }
        let angle = step_angle * i as f32;
        let cos = angle.cos();
        let sin = angle.sin();
        let mut line_vertex_index = 0;
        for d in distance.iter() {
            let cur_r = step_r[line_vertex_index];
            h += d;
            let p = [cur_r * cos, cur_r * sin, h, if line_vertex_index < 2 { 0.0 } else { 0.1 }];
            particles.push(BristleParticle { pos: p, old_pos: p, connect: [-1; 4] });
            index_data.push(index);
            index += 1;
            line_vertex_index += 1;
        }
        // 图元重启
        index_data.push((2_u64.pow(32) - 1) as u32);
    }
    // stretch constraints
    // 每个点的 stretch 取 上，右 两个为一组
    // 与邻居 bristle 的约束距离会越来近，形成笔尖

    let mut groups: Vec<Vec<[i32; 3]>> = vec![vec![], vec![], vec![], vec![]];
    for line in 0..bristle_num {
        for n in 2..distance.len() {
            let cur = line * distance.len() + n;
            // 最后一条bristle 与第一条形成约束
            let right = if line == (bristle_num - 1) { n } else { cur + distance.len() };
            // 将每个粒子对应的全部约束的索引装进单独的数组里
            let particle0 = &particles[cur];

            let mut group: [i32; 3] = [-1; 3];
            group[0] = constraints.len() as i32;
            constraints.push(StretchConstraint {
                rest_length: get_distance(&particle0, &particles[cur - 1]),
                particle0: cur as i32,
                particle1: cur as i32 - 1,
            });

            group[1] = constraints.len() as i32;
            constraints.push(StretchConstraint {
                rest_length: get_distance(&particle0, &particles[right]),
                particle0: cur as i32,
                particle1: right as i32,
            });
            // 剪切约束
            group[2] = constraints.len() as i32;
            constraints.push(StretchConstraint {
                rest_length: get_distance(&particle0, &particles[right - 1]),
                particle0: cur as i32,
                particle1: right as i32 - 1,
            });

            // 可归为4组，odd bristle(0, 1), even bristle(0, 1)
            let index = (line % 2) * 2 + n % 2;
            groups[index].push(group);
        }
    }

    let mut mesh_colorings: Vec<MeshColoringObj> = vec![];
    let mut reorder_stretches: Vec<[i32; 3]> = vec![];
    let mut offset = 0;
    for a_group in groups.iter_mut() {
        println!("stretch group len: {}", a_group.len());
        let group_len = a_group.len() as u32;
        mesh_colorings.push(MeshColoringObj {
            offset,
            max_num_x: 0,
            max_num_y: 0,
            group_len,
            thread_group: (((group_len + 31) as f32 / 32.0).floor() as u32, 1),
        });
        reorder_stretches.append(a_group);
        offset += group_len;
    }

    let (bend_mesh_colorings, bending_constraints, reorder_bendings) =
        cal_bend_constraints(bristle_num, distance.len());
    (
        particles,
        index_data,
        constraints,
        reorder_stretches,
        mesh_colorings,
        bending_constraints,
        reorder_bendings,
        bend_mesh_colorings,
    )
}

pub fn cal_bend_constraints(
    bristle_num: usize, vertex_num: usize,
) -> (Vec<MeshColoringObj>, Vec<BendingConstraintObj>, Vec<[i32; 2]>) {
    let mut bendings: Vec<BendingConstraintObj> = Vec::with_capacity(bristle_num * vertex_num * 2);
    let mut reorder_bendings: Vec<[i32; 2]> = vec![];

    let mut four_groups: Vec<Vec<[i32; 2]>> = vec![vec![], vec![], vec![], vec![]];
    let h0: f32 = 0.0;
    for line in 0..bristle_num {
        for n in 2..(vertex_num - 1) {
            let v = (line * vertex_num + n) as i32;
            // 垂直约束
            let b0 = v - 1;
            let b1 = v + 1;
            four_groups[n % 4].push([bendings.len() as i32, -1]);
            bendings.push(BendingConstraintObj { v, b0, b1, h0 });
        }
    }
    let mut mesh_colorings: Vec<MeshColoringObj> = vec![];
    let mut offset = 0;
    for a_group in four_groups.iter_mut() {
        println!("bend group len: {}", a_group.len());
        let group_len = a_group.len() as u32;
        mesh_colorings.push(MeshColoringObj {
            offset,
            max_num_x: 0,
            max_num_y: 0,
            group_len,
            thread_group: (((group_len + 31) as f32 / 32.0).floor() as u32, 1),
        });
        reorder_bendings.append(a_group);
        offset += group_len;
    }
    (mesh_colorings, bendings, reorder_bendings)
}

fn get_distance(particle0: &BristleParticle, particle1: &BristleParticle) -> f32 {
    let x = particle0.pos[0] - particle1.pos[0];
    let y = particle0.pos[1] - particle1.pos[1];
    let z = particle0.pos[2] - particle1.pos[2];

    (x * x + y * y + z * z).sqrt()
}
