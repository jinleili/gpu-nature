struct VertexOutput {
    @builtin(position) position: vec4<f32>,
};


struct MVPMatUniform {
    mv: mat4x4<f32>,
    proj: mat4x4<f32>,
    mvp: mat4x4<f32>,
    normal: mat4x4<f32>,
};

@group(0) @binding(0) var<uniform> mvp_mat: MVPMatUniform;

@stage(vertex)
fn vs_main(@builtin(vertex_index) vertexIndex: u32) -> VertexOutput {
    let uv: vec2<f32> = vec2<f32>(f32((vertexIndex << 1u) & 2u), f32(vertexIndex & 2u));

    var out: VertexOutput;
    out.position = mvp_mat.mvp * vec4<f32>(uv * 2.0 - 1.0, 0.0, 1.0);
 
    return out;
}

@stage(fragment)
fn fs_main() -> @location(0) vec4<f32> {
    var frag_color = vec4<f32>(vec3<f32>(0.95), 0.35);
    return frag_color;
}