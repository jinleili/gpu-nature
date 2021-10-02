#include "pbd/cloth_layout.wgsl"

let force: vec4<f32>  = vec4<f32>(0.0, 0.0, 0.0, 0.0);

let ball_pos: vec4<f32> = vec4<f32>(0.0, 0.0, 0.0, 0.0);

[[stage(compute), workgroup_size(32)]]
fn main([[builtin(global_invocation_id)]] global_invocation_id: vec3<u32>) {
    let total = arrayLength(&particles.data);
    let field_index = global_invocation_id.x;
    if (field_index >= total) {
      return;
    }
    var particle: ParticleObj = particles.data[field_index];
    if (is_movable_particle(particle)) {
      let temp_pos = particle.pos;
      // 预估新的位置
      particle.pos = particle.pos + (particle.pos - particle.old_pos) + (particle.accelerate + force) * cloth.dt * cloth.dt;
      particle.old_pos = temp_pos;
      particles.data[field_index] = particle;

      // 添加外力
      // vec4 dis_pos = particle.pos - ball_pos;
      // float dis = length(dis_pos);
      // if (dis < 0.9) {
      //   particle.pos += normalize(dis_pos) * (0.9 - dis);
      // }
    }

    // storage buffer item 里的字段如果有数组，那 item 只能使用 var 绑定，否则后面读取 item 字段时会报错：
    // The expression [73] may only be indexed by a constant
    var pConstraints = particle_constraints.data[field_index];
    // 重置所有约束
    for (var i: i32 = 0; i < 8; i = i + 1) {
      let constraint_index = pConstraints.list[i];
      if (constraint_index >= 0) {
        constraints.data[constraint_index].lambda = 0.0;
      }
    }
}