
struct TrajectoryUniform {
    screen_factor: vec2<f32>,
    // which view particles position will drawing to. 
    trajectory_view_index: i32,
    bg_view_index: i32,
};

@group(0) @binding(0) var<uniform> params: TrajectoryUniform;
@group(0) @binding(1) var macro_info: texture_3d<f32>;
@group(0) @binding(2) var tex_sampler: sampler;

struct VertexOutput {
    @location(0) uv: vec3<f32>;
    @builtin(position) position: vec4<f32>,
};

@vertex
fn vs_compose(
    @location(0) particle_pos: vec4<f32>,
    @location(1) particle_pos_initial: vec4<f32>,
    @location(2) position: vec2<f32>,
) -> VertexOutput {
    let pos = vec4<f32>(particle_pos.xy + position * params.screen_factor, particle_pos.z, particle_pos.w);
    var result: VertexOutput;
    result.position = pos;
    result.uv = (pos.xyz + 1.0) / 2.0;
    return result;
}

#include "func/color_space_convert.wgsl"

let PI: f32 = 3.1415926535;

@fragment 
fn fs_compose(in : VertexOutput) -> @location(0) vec4<f32> {
    let macro: vec4<f32> = textureSample(macro_info, tex_sampler, in.uv.xyz);
    let speed = abs(macro.x) + abs(macro.y);

     // moving angle as color
    let angle = (atan2(macro.y, macro.x) + PI) / (2.0 * PI);
    var frag_color: vec4<f32>;
    frag_color = vec4<f32>(hsv2rgb(angle , 0.9, 0.6 + speed * 2.0), macro.z);

    return frag_color;
    // return vec4<f32>(0.8);
}

