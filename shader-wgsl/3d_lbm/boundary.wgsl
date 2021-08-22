#include "3d_lbm/d3q15_layout_and_fn.wgsl"

[[stage(compute), workgroup_size(64)]]
fn main([[builtin(global_invocation_id)]] GlobalInvocationID: vec3<u32>) {
    let uv = vec3<i32>(GlobalInvocationID.xyz);
    var field_index : i32 = fieldIndex(uv);
    let info: LatticeInfo = lattice_info.data[field_index];
    if (isBoundaryCell(info.material) || isObstacleCell(info.material)) {
        // find lattice that direction quantities flowed in
        // push scheme: bounce back the direction quantities to that lattice
        // pull scheme: copy lattice reversed direction quantities to boundary cell
        for (var i : i32 = 0; i < 15; i = i + 1) {
            // lattice coords that will bounce back to
            let new_uv : vec3<i32> = uv - vec3<i32>(e(i));
            if (new_uv.x <= 0 || new_uv.y <= 0 || new_uv.z <= 0 || new_uv.x >= (field.lattice_size.x - 1) 
                || new_uv.y >= (field.lattice_size.y - 1) || new_uv.z >= (field.lattice_size.z - 1)) {
                continue;
            } else {
                // pull scheme:
                let lattice_index = latticeIndex(uv, REVERSED_DERECTION[i]);
                stream_cell.data[lattice_index] = stream_cell.data[latticeIndex(new_uv, i)];
            }
        }
    }
}