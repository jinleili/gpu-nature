struct VertexOutput {
    @location(0) uv: vec2<f32>,
    @builtin(position) position: vec4<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertexIndex: u32) -> VertexOutput {
    let uv: vec2<f32> = vec2<f32>(f32((vertexIndex << 1u) & 2u), f32(vertexIndex & 2u));
    var result: VertexOutput;
    result.position = vec4<f32>(uv * 2.0 - 1.0, 0.0, 1.0);
    // invert uv.y
    result.uv = vec2<f32>(uv.x, (uv.y - 1.0) *  (-1.0));
    return result;
}