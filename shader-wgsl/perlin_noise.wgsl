
// cpu version: https://mrl.cs.nyu.edu/~perlin/noise/
// Implementing Improved Perlin Noise:
// https://developer.nvidia.com/gpugems/gpugems2/part-iii-high-quality-rendering/chapter-26-implementing-improved-perlin-noise

#include "bufferless.vs.wgsl"

[[block]]
struct Permutation {
    data: [[stride(4)]] array<i32>;
};
[[block]]
struct Gradient {
    data: [[stride(16)]] array<vec4<f32>>;
};

[[group(0), binding(0)]] var<storage, read> permutation : Permutation;
[[group(0), binding(1)]] var<storage, read> gradient : Gradient;

let one: vec3<f32> = vec3<f32>(1.0, 1.0, 1.0);

fn fade(t: vec3<f32>) -> vec3<f32> {
    return t * t * t * (t * (t * 6.0 - 15.0) + 10.0);
}

fn perm(x: i32) -> i32 {
    // modf 不能操作标量!!（2021/8/10）
    // var float256: vec3<f32> = one * 256.0;
    // let index = i32(modf(one * x, &float256).x);
    // return f32(permutation.data[i32(x % 256.0)]);
    return permutation.data[x];
}

fn grad(x: i32, p: vec3<f32>) -> f32 {
    // var float16: vec3<f32> = one * 16.0;
    // let index = i32(modf(one * x, &float16).x);
    // return dot(gradient.data[index].xyz, p);
    return dot(gradient.data[x % 16].xyz, p);
}

fn lerp(a: f32, b: f32, w: f32) -> f32 {
    return a + (b - a) * w;
}

fn noise(pos: vec3<f32>) -> f32 {
    // find unit cube
    let P: vec3<i32> = vec3<i32>(floor(pos)) & vec3<i32>(255);  
    // point in unit cube
    let decimal_part_pos = pos - floor(pos);  
    // fade curves  
    let f: vec3<f32> = fade(decimal_part_pos);      
    // HASH COORDINATES FOR 6 OF THE 8 CUBE CORNERS      
    let A = perm(P.x) + P.y;    
    let AA = perm(A) + P.z;    
    let AB = perm(A + 1) + P.z;    
    let B =  perm(P.x + 1) + P.y;    
    let BA = perm(B) + P.z;    
    let BB = perm(B + 1) + P.z;  

    // AND ADD BLENDED RESULTS FROM 8 CORNERS OF CUBE  
    return lerp(lerp(lerp(grad(perm(AA), decimal_part_pos), grad(perm(BA), decimal_part_pos + vec3<f32>(-1.0, 0.0, 0.0)), f.x),           
    lerp(grad(perm(AB), decimal_part_pos + vec3<f32>(0.0, -1.0, 0.0)), grad(perm(BB), decimal_part_pos + vec3<f32>(-1.0, -1.0, 0.0)), f.x), f.y),      
    lerp(lerp(grad(perm(AA + 1), decimal_part_pos + vec3<f32>(0.0, 0.0, -1.0)), grad(perm(BA + 1), decimal_part_pos + vec3<f32>(-1.0, 0.0, -1.0)), f.x),           
    lerp(grad(perm(AB + 1), decimal_part_pos + vec3<f32>(0.0, -1.0, -1.0)), grad(perm(BB + 1), decimal_part_pos + vec3<f32>(-1.0, -1.0, -1.0)), f.x), f.y), f.z); 
}


[[stage(fragment)]] 
fn main([[builtin(position)]] coord : vec4<f32>) -> [[location(0)]] vec4<f32> {
    let p = coord.xy / 16.0; 
    return vec4<f32>(vec3<f32>(noise(vec3<f32>(p, 0.5))), 1.0);
}

