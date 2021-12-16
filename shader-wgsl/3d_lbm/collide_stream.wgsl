#include "3d_lbm/d3q15_layout_and_fn.wgsl"

let MAX_VALUE: array<f32, 15> = array<f32, 15>(0.3, 0.16666, 0.16666, 0.16666, 0.16666, 0.16666, 0.16666, 0.023, 0.023, 0.023, 0.023, 0.023, 0.023, 0.023, 0.023);

fn equilibrium(velocity: vec3<f32>, rho: f32, direction: i32, usqr: f32) -> f32 {
  let e_dot_u = dot(e(direction), velocity);
  // internal fn pow(x, y) requires x cannot be negative
  return rho * w(direction) * (1.0 + 3.0 * e_dot_u + 4.5 * (e_dot_u * e_dot_u) - usqr);
}


[[stage(compute), workgroup_size(64, 1, 1)]]
fn cs_main([[builtin(global_invocation_id)]] global_invocation_id: vec3<u32>) {
    let uv = vec3<i32>(global_invocation_id.xyz);
    var field_index : i32 = fieldIndex(uv);
    var info: LatticeInfo = lattice_info.data[field_index];
    // streaming out on boundary cell will cause crash
    if (isBoundaryCell(info.material) || isObstacleCell(info.material)) {
      textureStore(macro_info, uv, vec4<f32>(0.0, 0.0, 0.0, 0.0));
      return;
    }
    
    var f_i : array<f32, 15>;
    var velocity : vec3<f32> = vec3<f32>(0.0);
    var rho : f32 = 0.0;
    for (var i : i32 = 0; i < 15; i = i + 1) {
      f_i[i] = collide_cell.data[streaming_in(uv, i)];
      // f_i[i] = collide_cell.data[field_index + soaOffset(i)];
      rho = rho + f_i[i];
      velocity = velocity + e(i) * f_i[i];
    }
     // reset lbm field sometimes cause Nan?
    // external force sometimes can cause Inf.
    if (isOutletCell(info.material)) {
      rho = 1.0;
    } else {
      rho = clamp(rho, 0.6, 1.4);
    }

    velocity = velocity / rho;
    // external forcing
    var F : array<f32, 15> = array<f32, 15>(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    if (isAccelerateCell(info.material)) {
      if (info.block_iter > 0) {
        info.block_iter = info.block_iter - 1;
        if (info.block_iter == 0) {
          info.material = 1;
        }
      }
      lattice_info.data[field_index] = info;

      let force = vec3<f32>(info.vx, info.vy, 0.0);
      // velocity.x = velocity.x + force_x * 0.5 / rho;
      velocity = force * 0.5 / rho;

      for (var i : i32 = 0; i < 9; i = i + 1) {
        F[i] = w(i) * 3.0 * dot(e(i), force);
      }
    }
   
    // macro_info.data[field_index] = vec4<f32>(velocity.x, velocity.y, rho, 0.0);
    textureStore(macro_info, uv, vec4<f32>(velocity.x, velocity.y, rho, 1.0));

    let usqr = 1.5 * dot(velocity, velocity);
    for (var i : i32 = 0; i < 15; i = i + 1) {
      var temp_val: f32 = f_i[i] - fluid.omega * (f_i[i] - equilibrium(velocity, rho, i, usqr)) + F[i];
      if (temp_val > MAX_VALUE[i] || isInf(temp_val)) {
        temp_val = MAX_VALUE[i];
      } elseif (temp_val < 0.0) {
        temp_val = 0.0;
      }
      // stream_cell.data[streaming_out(uv, i)] = temp_val;
      stream_cell.data[latticeIndex(uv, i)] = temp_val;
    }
}
