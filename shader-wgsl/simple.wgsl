

struct MVPMatUniform {
    mv: mat4x4<f32>;
    proj: mat4x4<f32>;
    mvp: mat4x4<f32>;
    normal: mat4x4<f32>;
};

@group(0) @binding(0) var<uniform> mvp_mat: MVPMatUniform;

struct VertexOutput {
    @builtin(position) position: vec4<f32>;
};

@stage(vertex)
fn vs_main(
    @location(0) pos: vec3<f32>,
    @location(1) tangent: vec4<f32>,
) -> VertexOutput {
    var output: VertexOutput;
    output.position = mvp_mat.mvp * vec4<f32>(pos, 1.0);
    return output;
}


@stage(fragment)
fn fs_main() -> @location(0) vec4<f32> {
    var frag_color = vec4<f32>(vec3<f32>(0.95), 0.65);
    return frag_color;
}