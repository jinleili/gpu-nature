
// cpu version: https://mrl.cs.nyu.edu/~perlin/noise/
// Implementing Improved Perlin Noise:
// https://developer.nvidia.com/gpugems/gpugems2/part-iii-high-quality-rendering/chapter-26-implementing-improved-perlin-noise

fn fade(t: vec3<f32>) -> vec3<f32> {
    return t * t * t * (t * (t * 6.0 - 15.0) + 10.0);
}

fn perm(x: i32, y: i32) -> vec4<i32> {
    // modf 不能操作标量!!（2021/8/10）
    // let one: vec3<f32> = vec3<f32>(1.0, 1.0, 1.0);
    // var float256: vec3<f32> = one * 256.0;
    // let index = i32(modf(one * x, &float256).x);
    // return f32(permutation.data[i32(x % 256.0)]);
    return permutation.data[y * 256 + x];
}

fn grad(x: i32, p: vec3<f32>) -> f32 {
    // only use 16-pixels slightly improve performence.
    return dot(gradient.data[x & 15].xyz, p);
}

fn lerp(a: f32, b: f32, w: f32) -> f32 {
    return a + (b - a) * w;
}

fn lerp3(a: vec3<f32>, b: vec3<f32>, w: f32) -> vec3<f32> {
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
    let hash = (perm(P.x, P.y) + P.z) & vec4<i32>(255);
    // let A = perm(P.x) + P.y;    
    // let B =  perm(P.x + 1) + P.y;    
    // let AA = perm(A) + P.z;    
    // let AB = perm(A + 1) + P.z;    
    // let BA = perm(B) + P.z;    
    // let BB = perm(B + 1) + P.z;  

    // BLENDED RESULTS FROM 8 CORNERS OF CUBE  
    return lerp(
        lerp(lerp( grad(hash.x, decimal_part_pos), grad(hash.z, decimal_part_pos + vec3<f32>(-1.0, 0.0, 0.0)), f.x),           
            lerp( grad(hash.y, decimal_part_pos + vec3<f32>(0.0, -1.0, 0.0)), 
                grad(hash.w, decimal_part_pos + vec3<f32>(-1.0, -1.0, 0.0)), f.x), 
            f.y),      
        lerp(lerp(grad(hash.x + 1, decimal_part_pos + vec3<f32>(0.0, 0.0, -1.0)), grad(hash.z + 1, decimal_part_pos + vec3<f32>(-1.0, 0.0, -1.0)), f.x),           
            lerp( grad(hash.y + 1, decimal_part_pos + vec3<f32>(0.0, -1.0, -1.0)), 
                grad(hash.w + 1, decimal_part_pos + vec3<f32>(-1.0, -1.0, -1.0)), f.x), 
            f.y),
        f.z); 
}
