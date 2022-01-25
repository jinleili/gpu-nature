#include "struct/field.wgsl"

@group(0) @binding(0) var<uniform> field: FieldUniform;
@group(0) @binding(1) var<storage, read_write> fb: FieldBuffer;

fn field_index(uv: vec2<i32>) -> i32 {
   return uv.x + (uv.y * field.lattice_size.x);
}

fn get_velocity(p: vec2<i32>) -> vec2<f32> {
    #insert_code_segment
}

fn get_velocity0(p: vec2<i32>) -> vec2<f32> {
    var v: vec2<f32> = vec2<f32>(0.0, 0.0);
    // modf 函数目前还无法使用（05/03）
    // v.x = -2.0 * modf(f32(p.y / 60), 2.0) + 1.0;
    return v;
}

@stage(compute) @workgroup_size(16, 16)
fn cs_main(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    let uv = vec2<i32>(global_invocation_id.xy);
    if (uv.x >= field.lattice_size.x || uv.y >= field.lattice_size.y) {
        return;
    }
    let index = field_index(uv);
    fb.data[index] = vec4<f32>(get_velocity(uv), 0.0, 0.0);
}