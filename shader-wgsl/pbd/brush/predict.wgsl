#include "pbd/brush/struct/particle.wgsl"
#include "pbd/brush/struct/brush_uniform.wgsl"

[[group(0), binding(0)]] var<uniform> brush: BrushUniform;
[[group(0), binding(1)]] var<storage, read_write> particles: ParticlesBuffer;


struct DynamicUniform {
  // 需要更新粒子的偏移
  need_update_offset: i32;
};
[[group(1), binding(0)]] var<uniform> dy_uniform: DynamicUniform;

let force: vec3<f32>  = vec3<f32>(0.0, 0.0, 9.98);

[[stage(compute), workgroup_size(32, 1, 1)]]
fn cs_main([[builtin(global_invocation_id)]] global_invocation_id: vec3<u32>) {
    let total = arrayLength(&particles.data);
    let field_index = global_invocation_id.x;
    if (field_index >= total) {
        return;
    }
    // 笔刷的移动及提按只由固定位置的 particle 传导，避免整体受力导致的 bristle particle 位置抖动?
    // 或者，没有接触到纸面的 particle 都接受运笔的偏移？
    var particle: Particle = particles.data[field_index];
    let inverst_mass = particle.pos.w;
    var need_write_back = false;

    if (is_movable_particle(particle) == false) {
        // if (dy_uniform.need_update_offset == 1) {
        //     let offset = vec3<f32>(0.0001, 0.0, -0.003);
        //     particle.pos = vec4<f32>(particle.pos.xyz + offset, inverst_mass);
        //     particle.old_pos = vec4<f32>(particle.old_pos.xyz + offset, inverst_mass);
        //     need_write_back = true;
        // }
    } else {
        let temp_pos = particle.pos;

        // 预估新的位置
        // particle.pos = particle.pos + (particle.pos - particle.old_pos) + (particle.accelerate + force) * brush.dt * brush.dt;
        // Xn+1 = Xn + dt * Vn + dt^2 * M^-1 * F(Xn)
        // particle.pos = particle.pos + brush.dt * particle.accelerate + force * inverst_mass * brush.dt * brush.dt;
        var new_pos = particle.pos.xyz + (particle.pos.xyz - particle.old_pos.xyz) + force * inverst_mass * brush.dt * brush.dt;
        if (new_pos.z < 0.0) {
          new_pos.z = 0.0;
        }
        particle.pos = vec4<f32>(new_pos, inverst_mass);
        particle.old_pos = temp_pos;
        need_write_back = true;
    }

    if (need_write_back) {
        particles.data[field_index] = particle;
    }

}