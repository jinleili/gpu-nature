#include "pbd/cloth_layout.wgsl"


struct DynamicUniform {
  // 第一帧之后，需要更新粒子的速度
  need_update_velocity: i32,
};
@group(1) @binding(0) var<uniform> dy_uniform: DynamicUniform;


let force: vec4<f32>  = vec4<f32>(0.0, -29.98, 0.0, 0.0);
let ball_pos: vec4<f32> = vec4<f32>(0.0, 0.0, 0.0, 0.0);

@stage(compute) @workgroup_size(32, 1, 1)
fn cs_main(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    let total = arrayLength(&particles.data);
    let field_index = global_invocation_id.x;
    if (field_index >= total) {
      return;
    }
    var particle: Particle = particles.data[field_index];
    if (is_movable_particle(particle)) {
      let temp_pos = particle.pos;

    //   if (dy_uniform.need_update_velocity == 1) {
    //       particle.accelerate = (particle.pos - temp_pos) / cloth.dt;
    //   }
      // 预估新的位置
    //   particle.pos = particle.pos + (particle.pos - particle.old_pos) + (particle.accelerate + force) * cloth.dt * cloth.dt;
    // Xn+1 = Xn + dt * Vn + dt^2 * M^-1 * F(Xn)
        // particle.pos = particle.pos + cloth.dt * particle.accelerate + force * particle.uv_mass.z * cloth.dt * cloth.dt;
        particle.pos = particle.pos + (particle.pos - particle.old_pos) + force * particle.uv_mass.z * cloth.dt * cloth.dt ;

      particle.old_pos = temp_pos;

      particles.data[field_index] = particle;


      // 添加外力
      // vec4 dis_pos = particle.pos - ball_pos;
      // float dis = length(dis_pos);
      // if (dis < 0.9) {
      //   particle.pos += normalize(dis_pos) * (0.9 - dis);
      // }
    }

}