#include "pbd/brush/struct/particle.wgsl"
#include "pbd/brush/struct/brush_uniform.wgsl"

[[block]]
struct MVPMatUniform {
    mv: mat4x4<f32>;
    proj: mat4x4<f32>;
    mvp: mat4x4<f32>;
    normal: mat4x4<f32>;
};

[[group(0), binding(0)]] var<uniform> mvp_mat: MVPMatUniform;
[[group(0), binding(1)]] var<uniform> brush: BrushUniform;
[[group(0), binding(2)]] var<storage, read_write> particles: ParticlesBuffer;

struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(1)]] normal: vec3<f32>;
    [[location(2)]] ec_pos: vec3<f32>;
};

[[stage(vertex)]]
fn main([[builtin(vertex_index)]] vertex_index: u32) -> VertexOutput {
    // 1，找出对应编号的粒子，
    // 2，使用粒子的位置来计算顶点位置
    let p0 = particles.data[vertex_index];

    let particle1 = particles.data[p0.connect[0] ];
    let particle2 = particles.data[p0.connect[1] ];
    let particle3 = particles.data[p0.connect[2] ];
    let particle4 = particles.data[p0.connect[3] ];

    let mv_pos = mvp_mat.mv * vec4<f32>(p0.pos.xyz, 1.0);

    var output: VertexOutput;
    output.position = mvp_mat.proj * mv_pos;
    output.normal = (cross(particle2.pos.xyz - p0.pos.xyz, particle1.pos.xyz - p0.pos.xyz) +
                        cross(particle4.pos.xyz - p0.pos.xyz, particle3.pos.xyz - p0.pos.xyz)) / 2.0;
    output.ec_pos = mv_pos.xyz;
   
    return output;
}


let light_color = vec3<f32>(0.9, 0.9, 0.9);
let light_pos = vec3<f32>(-0.0, -0.0, 0.6);
let view_pos = vec3<f32>(0.0, 0.0, 1.0);

[[stage(fragment)]]
fn main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    let color: vec4<f32> = vec4<f32>(1.0);
    var norm = normalize(in.normal);
    // 利用 faceforward 函数的方法，判断面相对于光线的朝向，如果背面朝向光源，则要反转法线
    let bg_color = color.rgb;
    let d = dot(view_pos, norm);
    if (d < 0.0) {
        norm = -norm;
    }

    // // Diffuse
    // let light_dir = normalize(light_pos - in.ec_pos);
    // // 0.5 ambient
    // let diffuse = clamp(abs(dot(norm, light_dir)), 0.5, 1.0) * color.rgb;
    // return vec4<f32>(diffuse, color.a);
    return color;
}