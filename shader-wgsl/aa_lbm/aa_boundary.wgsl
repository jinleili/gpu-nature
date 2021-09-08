#include "aa_lbm/aa_layout_and_fn.wgsl"

[[stage(compute), workgroup_size(64, 4)]]
fn main([[builtin(global_invocation_id)]] global_invocation_id: vec3<u32>) {
    let uv = vec2<i32>(global_invocation_id.xy);
    if (uv.x >= field.lattice_size.x || uv.y >= field.lattice_size.y) {
      return;
    }
    var field_index : i32 = fieldIndex(uv);
    let info: LatticeInfo = lattice_info.data[field_index];
    if (isBoundaryCell(info.material) || isObstacleCell(info.material)) {
        for (var i : i32 = 0; i < 9; i = i + 1) {
            // lattice coords that will bounce back to
            let new_uv : vec2<i32> = uv + vec2<i32>(e(i));
            if (new_uv.x <= 0 || new_uv.y <= 0 || new_uv.x >= (field.lattice_size.x - 1) || new_uv.y >= (field.lattice_size.y - 1)) {
                continue;
            } else {
                let local_index = field_index + soaOffset(i);
                let bounce_back_index = latticeIndex(new_uv, fluid.inversed_direction[i].x);
                // aa_cell.data[bounce_back_index] = aa_cell.data[bounce_back_index] + aa_cell.data[local_index];
                aa_cell.data[bounce_back_index] =  aa_cell.data[local_index];
                aa_cell.data[local_index] = 0.0;
            }
        }
    }
}