

#include "bufferless.vs.wgsl"


struct Permutation {
    data: array<vec4<i32>>,
};

struct Gradient {
    data: array<vec4<f32>>,
};

@group(0) @binding(0) var<storage, read> permutation : Permutation;
@group(0) @binding(1) var<storage, read> gradient : Gradient;

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

@fragment 
fn fs_main(@builtin(position) coord : vec4<f32>) -> @location(0) vec4<f32> {
    let p = vec3<f32>(coord.xy / 105.0, 0.5) ; 
    // noise self
    // return vec4<f32>(vec3<f32>(noise(p)), 1.0);

    // marble
    // 绿底白纹
    let color1 = vec3<f32>(0.1, 0.85, 0.2);
    let color2 = vec3<f32>(0.88);
    let marble = lerp3(color1, color2, cos(p.z * 0.1 + 6.0 * turbulence(5, p, 1.9, 0.55)));
    return vec4<f32>(marble, 1.0); 
}

