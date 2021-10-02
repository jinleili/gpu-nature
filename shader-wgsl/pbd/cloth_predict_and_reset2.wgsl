#include "pbd/cloth_layout2.wgsl"


let ball_pos: vec4<f32> = vec4<f32>(0.0, 0.0, 0.0, 0.0);

[[stage(compute), workgroup_size(16, 16)]]
fn main([[builtin(global_invocation_id)]] global_invocation_id: vec3<u32>) {
  let uv = vec2<i32>(global_invocation_id.xy);
  if (uv.x >= cloth.num_x || uv.y >= cloth.num_y) {
    return;
  }
  let particle_index: i32 = uv.y * cloth.num_x + uv.x;
  var particle: ParticleObj = particles.data[particle_index];
  if (is_movable_particle(particle)) {
    let temp_pos = particle.pos;
    // 预估新的位置
    particle.pos = particle.pos + (particle.pos - particle.old_pos) + particle.accelerate * cloth.dt * cloth.dt;
    particle.old_pos = temp_pos;
    particles.data[particle_index] = particle;

    // 测试：将最后一排的顶点直接碰撞到布料中心位置
    // if (uv.y == num_y - 1) {
    //   int other_index = (num_y / 2) * num_x + uv.x;
    //   vec4 pos = particles[other_index].pos;
    //   particle.old_pos = pos;
    //   particle.old_pos.y += 0.005;
    //   particle.pos = pos;
    //   particle.pos.y -= 0.03;

    //   particles[particle_index] = particle;
    // }

    // 添加外力
    // vec4 dis_pos = particle.pos - ball_pos;
    // float dis = length(dis_pos);
    // if (dis < 0.9) {
    //   particle.pos += normalize(dis_pos) * (0.9 - dis);
    // }
  }
  // 重置 bin
  let bin_index: i32 = linearize_bin_index(particle.pos.xyz);
  if (bin_index != -1) {
    bins.data[bin_index].list = array<i32, 16>(frame.frame_index, 0, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1);
  }
  // 清空碰撞约束
  var collision: CollisionObj = collisions.data[particle_index];
  collision.count = 0;
  collision.triangles = array<i32, 8>(-1, -1, -1, -1, -1, -1, -1, -1);
  collisions.data[particle_index] = collision;

  var particle_constraints = particle_constraints.data[particle_index];
  // 重置stretch约束
  for (var i: i32 = 0; i < 3; i = i + 1) {
    let constraint_index = particle_constraints.stretches[i];
    if (constraint_index >= 0) {
      stretches.data[constraint_index].lambda = 0.0;
    }
  }
}