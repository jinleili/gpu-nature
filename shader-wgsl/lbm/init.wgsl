#include "lbm/struct/lbm_uniform.wgsl"
#include "lbm/struct/lattice_info.wgsl"
#include "struct/field.wgsl"

[[block]]
struct StoreFloat {
    data: [[stride(4)]] array<f32>;
};

[[group(0), binding(0)]] var<uniform> fluid: LbmUniform;
[[group(0), binding(1)]] var<uniform> field: FieldUniform;
[[group(0), binding(2)]] var<storage, read_write> collide_cell: StoreFloat;
[[group(0), binding(3)]] var<storage, read_write> stream_cell: StoreFloat;
[[group(0), binding(4)]] var<storage, read_write> lattice_info: StoreInfo;
[[group(0), binding(5)]] var macro_info: texture_storage_2d<rgba16float, write>;

#include "lbm/d2q9_fn.wgsl"

[[stage(compute), workgroup_size(64, 4)]]
fn main([[builtin(global_invocation_id)]] GlobalInvocationID: vec3<u32>) {
  let uv = vec2<i32>(GlobalInvocationID.xy);
  if (uv.x >= field.lattice_size.x || uv.y >= field.lattice_size.y) {
    return;
  }
  let field_index = fieldIndex(uv);
  
  var info: LatticeInfo = lattice_info.data[field_index];
  if (isBoundaryCell(info.material) || isObstacleCell(info.material)) {
    for (var i : i32 = 0; i < 9; i = i + 1) {
        // lattice coords that will bounce back to
        let new_uv : vec2<i32> = uv - vec2<i32>(e(i));
        if (new_uv.x <= 0 || new_uv.y <= 0 || new_uv.x >= (field.lattice_size.x - 1) || new_uv.y >= (field.lattice_size.y - 1)) {
            collide_cell.data[field_index + soaOffset(i)] =  0.0;
            stream_cell.data[field_index + soaOffset(i)] = 0.0;
        } else {
            // pull scheme:
            let new_index = field_index + soaOffset(fluid.inversed_direction[i].x);
            collide_cell.data[new_index] =  w(i);
            stream_cell.data[new_index] = w(i);
        }
    }
  } elseif (isPoiseuilleFlow()) {
    for (var i: i32 = 0; i < 9; i = i + 1) {
      collide_cell.data[field_index + soaOffset(i)] =  w(i);
      stream_cell.data[field_index + soaOffset(i)] = 0.0;
    }
    let temp = w(3) * 0.5;
    collide_cell.data[field_index + soaOffset(1)] = w(1) + temp;
    collide_cell.data[field_index + soaOffset(3)] = temp;
    stream_cell.data[field_index + soaOffset(1)] =  w(1) + temp;
    stream_cell.data[field_index + soaOffset(3)] = temp;
  } else {
    for (var i: i32 = 0; i < 9; i = i + 1) {
      collide_cell.data[field_index + soaOffset(i)] =  w(i);
      collide_cell.data[field_index + soaOffset(i)] =  0.0;
    }
  }

  if (isAccelerateCell(info.material)) {
    if (info.block_iter > 0) {
        info.block_iter = 0;
        info.material = 1;
        info.vx = 0.0;
        info.vy = 0.0;
        lattice_info.data[field_index] = info;
      }
  }

  // macro_info.data[field_index] = vec4<f32>(0.0, 0.0, 0.0, 0.0);
  textureStore(macro_info, vec2<i32>(uv), vec4<f32>(0.0, 0.0, 0.0, 1.0));
}