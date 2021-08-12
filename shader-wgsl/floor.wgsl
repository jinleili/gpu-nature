
[[block]]
struct MVPMatUniform {
    mvp: mat4x4<f32>;
};

[[group(0), binding(0)]] var<uniform> mvp_mat: MVPMatUniform;

struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(0)]] uv: vec2<f32>;
};

[[stage(vertex)]]
fn main(
    [[location(0)]] pos: vec3<f32>,
    [[location(1)]] uv: vec2<f32>
) -> VertexOutput {
    var output: VertexOutput;
    output.position = mvp_mat.mvp * vec4<f32>(pos, 1.0);
    output.uv = uv;
    return output;
}


[[group(0), binding(1)]] var tex: texture_2d<f32>;
[[group(0), binding(2)]] var tex_sampler: sampler;

[[stage(fragment)]]
fn main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    let color: vec4<f32> = textureSample(tex, tex_sampler, in.uv);

    return color;
}