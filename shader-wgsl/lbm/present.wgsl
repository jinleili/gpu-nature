#include "bufferless.vs.wgsl"

#include "struct/field.wgsl"
#include "struct/particle.wgsl"
#include "struct/canvas.wgsl"

[[group(0), binding(0)]] var<uniform> field: FieldUniform;
[[group(0), binding(1)]] var<uniform> particle_uniform: ParticleUniform;
[[group(0), binding(2)]] var<storage, read_write> canvas: CanvasBuffer;
[[group(0), binding(3)]] var macro_info: texture_2d<f32>;
[[group(0), binding(4)]] var cur_info: texture_2d<f32>;
[[group(0), binding(5)]] var tex_sampler: sampler;

#include "func/color_space_convert.wgsl"

let PI: f32 = 3.1415926535;

[[stage(fragment)]] 
fn main(in : VertexOutput) -> [[location(0)]] vec4<f32> {
    //trick wgpu validation
    let xx = particle_uniform.color;
    let pixel_coord = min(vec2<i32>(floor(in.position.xy)), field.canvas_size.xy - 1);
    let p_index = pixel_coord.x + pixel_coord.y * field.canvas_size.x;
    var p: Pixel = canvas.pixels[p_index];
    let macro: vec4<f32> = textureSample(macro_info, tex_sampler, in.uv);

    // calculate curl
    let curl: vec4<f32> = textureSample(cur_info, tex_sampler, in.uv);

    var frag_color: vec4<f32>;
    let speed = abs(macro.x) + abs(macro.y);

     // moving angle as color
    let angle = (atan2(macro.y, macro.x) + PI) / (2.0 * PI);
    // frag_color = vec4<f32>(hsv2rgb(angle, 0.9, 1.0), macro.z);
    // frag_color = vec4<f32>(hsv2rgb(curl.x , 0.9, 0.6 + speed * 2.0), macro.z);
    frag_color = vec4<f32>(hsv2rgb(curl.x , 0.6 + speed * 1.4, 0.6 + macro.z * 0.33), macro.z);

    return frag_color;
}

