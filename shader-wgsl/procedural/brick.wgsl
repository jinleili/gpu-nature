#include "bufferless.vs.wgsl"

struct BrickParams {
    wh: vec2<f32>,
    mortar_thickness: f32,
    // brick + mortar thickness
    bm_wh: vec2<f32>,
    // mortar half width and height within the brick
    mortar_half_wh: vec2<f32>,
};

@group(0) @binding(0) var<uniform> params: BrickParams;

let brick_color: vec4<f32> = vec4<f32>(0.5, 0.15, 0.14, 1.0);
let mortar_color: vec4<f32> = vec4<f32>(0.5, 0.5, 0.5, 1.0);
let mortar_dark_color: vec4<f32> = vec4<f32>(0.30, 0.30, 0.30, 1.0);

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    var st: vec2<f32> = (vertex.position.xy + params.mortar_thickness * 0.5)/ params.bm_wh;
    // naga 中的 modf 现在还需要使用指针，用 fract 替代更简捷
    // var one: f32 = 1.0;
    // if (modf(st.y * 0.5, &one) > 0.5) {
    if (fract(st.y * 0.5) > 0.5) {
        // offset alternate rows
        st.x += 0.5;
    }

    // which brick
    let brick = floor(st);
    // coordinates within the brick
    st -= brick;
    let wh = step(params.mortar_half_wh, st) - step(vec2<f32>(1.0) - params.mortar_half_wh, st);
    // compute bump for mortar grooves
    let bump = smoothstep(vec2<f32>(0.0), params.mortar_half_wh, st) - smoothstep(vec2<f32>(1.0) - params.mortar_half_wh, vec2<f32>(1.0), st);
    let mixed_mortar_color = mix(mortar_dark_color, mortar_color, bump.x * bump.y);
    return mix(mixed_mortar_color, brick_color, wh.x * wh.y);
}