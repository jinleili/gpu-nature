

struct Permutation {
    data: array<vec4<i32>>,
};

struct Gradient {
    data: array<vec4<f32>>,
};

@group(0) @binding(0) var<storage, read> permutation : Permutation;
@group(0) @binding(1) var<storage, read> gradient : Gradient;
@group(0) @binding(2) var tex: texture_storage_3d<rgba8unorm, write>;

#include "noise/fn_perlin_noise.wgsl"


@stage(compute) @workgroup_size(8, 8, 8)
fn cs_main(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    let p = vec2<f32>(global_invocation_id.xyz) / 8.0 ; 
    let val = noise(p);
    
    textureStore(tex, vec3<i32>(global_invocation_id.xyz), vec4<f32>(val, val * 0.5 + 0.5, val * 0.25 + 0.5, val * 0.125 + 0.5));
}

