use super::particle::{ConstraintBufferObj, ParticleBufferObj};
use idroid::math::Point3D;
use zerocopy::{AsBytes, FromBytes};

pub fn generate_line_particles(
    horizontal_num: usize, vertical_num: usize, horizontal_pixel: f32, vertical_pixel: f32, a_pixel_on_ndc: f32,
) -> (Vec<ParticleBufferObj>, Vec<ConstraintBufferObj>) {
    let mut particles = Vec::with_capacity(horizontal_num * vertical_num);
    let mut constraints = Vec::with_capacity(horizontal_num * vertical_num);

    let line_gap = horizontal_pixel / (horizontal_num - 1) as f32 * a_pixel_on_ndc;
    let vertical_step = vertical_pixel / (vertical_num - 1) as f32 * a_pixel_on_ndc;

    // 第一条线的起始处
    let line_start_x = (-line_gap) * ((horizontal_num - 1) as f32 / 2.0);
    let tl_y = vertical_step * ((vertical_num - 1) as f32 / 2.0);
    let mut invert_mass = 0.1;
    // 按线的顺序遍历
    for line_index in 0..horizontal_num {
        let py = 1.2;
        let px = line_start_x + line_gap * line_index as f32;
        for h in 0..vertical_num {
            // 线默认横着放
            let p = [px - vertical_step * h as f32, py, 0.0, 0.0];
            // 线的起点固定：粒子质量为 0
            if h <= 0 {
                invert_mass = 0.0;
            } else {
                invert_mass = 0.1;
            }
            particles.push(ParticleBufferObj {
                pos: p,
                old_pos: p,
                accelerate: [0.0, -0.98, 0.0, 0.0],
                // webgpu 的纹理坐标是左上角为 0，0
                uv_mass: [0.0, 0.0, invert_mass, 0.0],
                connect: [0; 4],
            });
            // print!("{:?}", [uv_x_step * w as f32, 1.0 - uv_y_step * h as f32]);
        }
    }
    // 将每个粒子对应的全部约束的索引装进单独的数组里
    for line_index in 0..horizontal_num {
        for h in 0..vertical_num {
            let index = line_index * vertical_num + h;
            let particle0 = &particles[index];
            let p0: Point3D = Point3D::new(particle0.pos[0], particle0.pos[1], particle0.pos[2]);
            if h < vertical_num - 1 {
                let particle1 = &particles[index + 1];
                let p1: Point3D = Point3D::new(particle1.pos[0], particle1.pos[1], particle1.pos[2]);
                let rest_length: f32 = p0.minus(&p1).length();
                constraints.push(ConstraintBufferObj {
                    rest_length,
                    lambda: 0.0,
                    particle0: index as i32,
                    particle1: 1 + index as i32,
                });
            } else {
                // 插入一个无效约束来保持约束与粒子一一对应
                constraints.push(ConstraintBufferObj { rest_length: 0.0, lambda: 0.0, particle0: -1, particle1: -1 });
            }
        }
    }
    (particles, constraints)
}
