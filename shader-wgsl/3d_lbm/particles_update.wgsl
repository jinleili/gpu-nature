#include "3d_lbm/struct/3d_particle.wgsl"

[[block]]
struct TrajectoryUniform {
    screen_factor: vec2<f32>;
    // which view particles position will drawing to. 
    trajectory_view_index: i32;
    bg_view_index: i32;
};

[[group(0), binding(0)]] var<uniform> params: TrajectoryUniform;
[[group(0), binding(1)]] var macro_info: texture_3d<f32>;
[[group(0), binding(2)]] var tex_sampler: sampler;
[[group(0), binding(3)]] var<storage, read_write> pb: ParticleBuffer;

struct VertexOutput {
    [[location(0)]] particle_index: i32;
    [[builtin(position)]] position: vec4<f32>;
};

[[stage(vertex)]]
fn vs_update(
      [[builtin(instance_index)]] inst_index: u32,
) -> VertexOutput {
    var output: VertexOutput;
    output.position = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    output.particle_index = i32(inst_index);
    return output;
    // return vec4<f32>(1.0);
}

[[stage(fragment)]] 
fn fs_update(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    var particle: Particle3D = pb.particles[in.particle_index];
    let uv = (particle.pos.xyz + 1.0) / 2.0;
    let macro: vec4<f32> = textureSample(macro_info, tex_sampler, uv);
    if (macro.z > 0.001) {
        // particle.pos = vec4<f32>(particle.pos.xyz + (macro.xyz * 4.0) * vec3<f32>(params.screen_factor, params.screen_factor.x), 1.0);
        pb.particles[in.particle_index].pos = vec4<f32>(particle.pos.xyz + (macro.xyz * 4.0) * vec3<f32>(params.screen_factor, params.screen_factor.x), 1.0);

        discard;
    } 
    
    return vec4<f32>(0.0);
}

