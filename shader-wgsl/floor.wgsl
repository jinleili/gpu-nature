

struct MVPMatUniform {
    mv: mat4x4<f32>,
    proj: mat4x4<f32>,
    mvp: mat4x4<f32>,
    normal: mat4x4<f32>,
};

@group(0) @binding(0) var<uniform> mvp_mat: MVPMatUniform;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) ec_pos: vec3<f32>,
    @location(2) mc_pos: vec3<f32>,
};

@vertex
fn vs_main(
    @location(0) pos: vec3<f32>,
    @location(1) uv: vec2<f32>,
) -> VertexOutput {
    var result: VertexOutput;
    result.position = mvp_mat.mvp * vec4<f32>(pos, 1.0);
    result.uv = uv;
    result.ec_pos = (mvp_mat.mv * vec4<f32>(pos, 1.0)).xyz;
    result.mc_pos = pos;
    return result;
}


@group(0) @binding(1) var tex: texture_2d<f32>;
@group(0) @binding(2) var noise: texture_3d<f32>;
@group(0) @binding(3) var tex_sampler: sampler;
@group(0) @binding(4) var noise_sampler: sampler;

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    let color: vec4<f32> = textureSample(tex, tex_sampler, vertex.uv);

    let nv: vec4<f32> = textureSample(noise, noise_sampler, vertex.mc_pos);
    var size = nv.r + nv.g + nv.b + nv.a; // [1.,3.] 
    let deltaz = (size / 3.0) * 0.95;
    var fogFactor: f32 = 0.0;
    if (vertex.mc_pos.y > -0.7) {
        fogFactor = (0.7 + vertex.mc_pos.y) * deltaz; 
        fogFactor = clamp( fogFactor, 0.0, 1.0 ); 
        fogFactor = smoothstep(0., 1., fogFactor);
    }
    return mix(color, vec4<f32>(0.0), fogFactor);
}