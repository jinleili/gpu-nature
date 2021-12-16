#include "pbd/struct/particle.wgsl"
#include "pbd/struct/collision.wgsl"
#include "pbd/struct/cloth_uniform.wgsl"


struct MVPMatUniform {
    mv: mat4x4<f32>;
    proj: mat4x4<f32>;
    mvp: mat4x4<f32>;
    normal: mat4x4<f32>;
};

[[group(0), binding(0)]] var<uniform> mvp_mat: MVPMatUniform;
[[group(0), binding(1)]] var<uniform> cloth: ClothUniform;
[[group(0), binding(2)]] var<storage, read_write> particles: ParticlesBuffer;
// [[group(0), binding(3)]] var<storage, read_write> collisions: CollisionObjBuf;

struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(0)]] uv: vec2<f32>;
    [[location(1)]] normal: vec3<f32>;
    [[location(2)]] ec_pos: vec3<f32>;
    [[location(3)]] collision_area: f32;
};

[[stage(vertex)]]
fn vs_main(
    [[location(0)]] particle_index: vec3<u32>,
) -> VertexOutput {
    let field_index = particle_index.x + particle_index.y * u32(cloth.num_x);
    // 1，找出对应编号的粒子，
    // 2，使用粒子的位置来计算顶点位置
    let particle = particles.data[field_index];

    let particle1 = particles.data[particle.connect[0] ];
    let particle2 = particles.data[particle.connect[1] ];
    let particle3 = particles.data[particle.connect[2] ];
    let particle4 = particles.data[particle.connect[3] ];

    let mv_pos = mvp_mat.mv * vec4<f32>(particle.pos.xyz, 1.0);

    var output: VertexOutput;
    // normal = normalize(cross(particle1.pos.xyz - particle.pos.xyz, particle2.pos.xyz - particle.pos.xyz) +
    //                    cross(particle3.pos.xyz - particle.pos.xyz, particle4.pos.xyz - particle.pos.xyz));
    output.normal = (cross(particle2.pos.xyz - particle.pos.xyz, particle1.pos.xyz - particle.pos.xyz) +
                        cross(particle4.pos.xyz - particle.pos.xyz, particle3.pos.xyz - particle.pos.xyz)) / 2.0;
    output.position = mvp_mat.proj * mv_pos;
    output.ec_pos = mv_pos.xyz;
    output.uv = particle.uv_mass.xy;
    output.collision_area = 0.0;
    // let collesion = collisions.data[field_index];
    // if (collesion.count > 0) {
    //     output.collision_area = 1.0;
    // } else {
    //     output.collision_area = 0.0;
    // }
   
    return output;
}

[[group(0), binding(3)]] var tex: texture_2d<f32>;
[[group(0), binding(4)]] var tex_sampler: sampler;

let light_color = vec3<f32>(0.9, 0.9, 0.9);
let light_pos = vec3<f32>(-0.0, -0.0, 0.6);
let view_pos = vec3<f32>(0.0, 0.0, 1.0);
let ambient_strength = 0.75;
let specular_strength = 0.05;

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    let color: vec4<f32> = textureSample(tex, tex_sampler, in.uv);
    let ambient = ambient_strength * light_color.rgb;
    // Diffuse
    var norm = normalize(in.normal);
    // 利用 faceforward 函数的方法，判断面相对于光线的朝向，如果背面朝向光源，则要反转法线
    let bg_color = color.rgb;
    let d = dot(view_pos, norm);
    if (d < 0.0) {
        norm = -norm;
    }

    let light_dir = normalize(light_pos - in.ec_pos);
    // 0.5 ambient
    let diffuse = clamp(abs(dot(norm, light_dir)), 0.5, 1.0) * color.rgb;
    return vec4<f32>(diffuse, color.a);

    // // Specular
    // let reflectDir = reflect(-lightDir, norm);
    // // 2, 4, 8, 16, 32,取值大，高光区越聚集
    // let spec = pow(max(dot(normalize(view_pos.xyz - in.ec_pos), reflectDir), 0.0), 2.0);
    // let specular = specular_strength * spec * light_color.rgb;

    // return vec4<f32>((ambient + diffuse + specular) * (color.rgb * 0.5 + bg_color * 0.5), color.a);
    // return vec4<f32>(1.0);
}