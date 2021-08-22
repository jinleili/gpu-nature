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
[[group(0), binding(5)]] var macro_info: texture_storage_3d<rgba16float, write>;

//  D3Q15 lattice direction coordinate:
// - 2 -             9 - 8           13 - 12
// 3 0 1   + front:  - 5 -   + back:  - 6 -
// - 4 -            10 - 7           14 - 11
let E: array<vec3<f32>, 15> = array<vec3<f32>, 15>(vec3<f32>(0.0, 0.0, 0.0), 
vec3<f32>(1.0, 0.0, 0.0), vec3<f32>(0.0, -1.0, 0.0), vec3<f32>(-1.0, 0.0, 0.0), vec3<f32>(0.0, 1.0, 0.0), 
vec3<f32>(0.0, 0.0, 1.0), vec3<f32>(0.0, 0.0, -1.0),
vec3<f32>(1.0, 1.0, 1.0), vec3<f32>(1.0, -1.0, 1.0), vec3<f32>(-1.0, -1.0, 1.0), vec3<f32>(-1.0, 1.0, 1.0),
vec3<f32>(1.0, 1.0, -1.0), vec3<f32>(1.0, -1.0, -1.0), vec3<f32>(-1.0, -1.0, -1.0), vec3<f32>(-1.0, 1.0, -1.0));

// lattice direction's weight: 2/9, 1/9, 1/72
let W: array<f32, 15> = array<f32, 15>(0.22222222, 0.11111111, 0.11111111, 0.11111111, 0.11111111, 0.11111111, 0.11111111,
 0.013888888, 0.013888888, 0.013888888, 0.013888888, 0.013888888, 0.013888888, 0.013888888, 0.013888888);

let REVERSED_DERECTION: array<i32, 15> = array<i32, 15>(0, 3, 4, 1, 2, 6, 5, 13, 14, 11, 12, 9, 10, 7, 8);

fn isPoiseuilleFlow() -> bool { return fluid.fluid_ty == 0; }

// direction's coordinate
fn e(direction: i32) -> vec3<f32> { return E[direction]; }
// direction's weight
fn w(direction: i32) -> f32 { return W[direction]; }

fn fieldIndex(uv: vec3<i32>) -> i32 { 
  return uv.x + (uv.y * field.lattice_size.x) + uv.z * (field.lattice_size.x * field.lattice_size.y); 
}
fn soaOffset(direction: i32) -> i32 { return direction * fluid.soa_offset; }
fn latticeIndex(uv: vec3<i32>, direction: i32) -> i32 {
  return fieldIndex(uv) + soaOffset(direction);
}

fn isBoundaryCell(material: i32) -> bool { return material == 2; }
fn isNotBoundaryCell(material: i32) -> bool { return material != 2; }
fn isAccelerateCell(material: i32) -> bool { return material == 3; }
fn isObstacleCell(material: i32) -> bool { return material == 4; }
fn isOutletCell(material: i32) -> bool { return material == 5; }

fn isBulkFluidCell(material: i32) -> bool { return material == 1 || material == 5 || material == 6; }

// pull scheme
fn streaming_in(uv : vec3<i32>, direction : i32) -> i32 {
    var target_uv : vec3<i32> = uv + vec3<i32>(e(REVERSED_DERECTION[direction]));
     if (target_uv.x < 0) {
      target_uv.x = field.lattice_size.x - 1;
    } elseif (target_uv.x >= field.lattice_size.x) {
      target_uv.x = 0;
    }
    if (target_uv.y < 0) {
      target_uv.y = field.lattice_size.y - 1;
    } elseif (target_uv.y >= field.lattice_size.y) {
      target_uv.y = 0;
    } 
    if (target_uv.z < 0) {
      target_uv.z = field.lattice_size.z - 1;
    } elseif (target_uv.z >= field.lattice_size.z) {
      target_uv.z = 0;
    } 
    return latticeIndex(target_uv, direction);
}