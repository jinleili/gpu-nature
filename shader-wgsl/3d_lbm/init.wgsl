#include "3d_lbm/d3q15_layout_and_fn.wgsl"


[[stage(compute), workgroup_size(64, 1, 1)]]
fn cs_main([[builtin(global_invocation_id)]] global_invocation_id: vec3<u32>) {
  let uv = vec3<i32>(global_invocation_id.xyz);

  let field_index = fieldIndex(uv);
  if (field_index < 10000) {
    var info: LatticeInfo = lattice_info.data[field_index];
    info.material = uv.x;
    info.block_iter = uv.y;
    info.vx = f32(uv.z);
    lattice_info.data[field_index] = info;
  }
  

  // if (isBoundaryCell(info.material) || isObstacleCell(info.material)) {
    // for (var i: i32 = 0; i < 9; i = i + 1) {
    // }
    // for (var i : i32 = 0; i < 15; i = i + 1) {
        // lattice coords that will bounce back to
        // let new_uv: vec3<i32> = uv - vec3<i32>(0, 0, 0);
        // if (new_uv.x <= 0 || new_uv.y <= 0 || new_uv.x >= (field.lattice_size.x - 1) || new_uv.y >= (field.lattice_size.y - 1)) {
        //     collide_cell.data[latticeIndex(uv, i)] =  0.0;
        //     stream_cell.data[latticeIndex(uv, i)] = 0.0;
        // } 
    //      else {
    //         // pull scheme:
    //         let new_index = latticeIndex(uv, REVERSED_DERECTION[i]);
    //         collide_cell.data[new_index] =  w(i);
    //         stream_cell.data[new_index] = w(i);
    //     }
    // }
  // } 
  // elseif (isPoiseuilleFlow()) {
  //   for (var i: i32 = 0; i < 15; i = i + 1) {
  //     collide_cell.data[latticeIndex(uv, i)] =  w(i);
  //     stream_cell.data[latticeIndex(uv, i)] = 0.0;
  //   }
  //   let temp = w(3) * 0.5;
  //   collide_cell.data[latticeIndex(uv, 1)] = w(1) + temp;
  //   collide_cell.data[latticeIndex(uv, 3)] = temp;
  //   stream_cell.data[latticeIndex(uv, 1)] =  w(1) + temp;
  //   stream_cell.data[latticeIndex(uv, 3)] = temp;
  // } else {
  //   for (var i: i32 = 0; i < 15; i = i + 1) {
  //     collide_cell.data[latticeIndex(uv, i)] =  w(i);
  //     collide_cell.data[latticeIndex(uv, i)] =  0.0;
  //   }
  // }

  // if (isAccelerateCell(info.material)) {
  //   if (info.block_iter > 0) {
  //       info.block_iter = 0;
  //       info.material = 1;
  //       info.vx = 0.0;
  //       info.vy = 0.0;
        // lattice_info.data[field_index] = info;
  //     }
  // }

  // textureStore(macro_info, uv, vec4<f32>(0.0, 0.0, 0.0, 1.0));
}