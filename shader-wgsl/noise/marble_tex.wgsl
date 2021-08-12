
[[block]]
struct Permutation {
    data: [[stride(16)]] array<vec4<i32>>;
};
[[block]]
struct Gradient {
    data: [[stride(16)]] array<vec4<f32>>;
};

[[group(0), binding(0)]] var<storage, read> permutation : Permutation;
[[group(0), binding(1)]] var<storage, read> gradient : Gradient;
[[group(0), binding(2)]] var marble_tex: texture_storage_2d<rgba8unorm, write>;

#include "noise/fn_perlin_noise.wgsl"

fn turbulence(octaves: i32, P: vec3<f32>, lacunarity: f32, gain: f32) -> f32 {	
  var sum: f32 = 0.0;
  var scale: f32 = 1.0;
  var totalgain: f32 = 1.0;
  for(var i = 0; i < octaves; i = i + 1){
    sum = sum + totalgain * noise(P * scale);
    scale = scale * lacunarity;
    totalgain = totalgain * gain;
  }
  return abs(sum);
}

[[stage(compute), workgroup_size(16, 16)]]
fn main([[builtin(global_invocation_id)]] global_invocation_id: vec3<u32>) {
    let xy = vec2<f32>(global_invocation_id.xy);
    let p = vec3<f32>(xy / 105.0, 0.5) ; 
    // marble
    // 绿底白纹
    let color1 = vec3<f32>(0.1, 0.85, 0.2);
    let color2 = vec3<f32>(0.88);
    let marble = lerp3(color1, color2, cos(p.z * 0.1 + 6.0 * turbulence(5, p, 1.9, 0.55)));
    textureStore(marble_tex, vec2<i32>(global_invocation_id.xy), vec4<f32>(marble, 1.0));
}

