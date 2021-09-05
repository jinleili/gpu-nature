#include "aa_lbm/aa_layout_and_fn.wgsl"

[[block]]
struct TickTockUniforms {
  // A-A pattern lattice offset
  read_offset: [[stride(4)]] array<i32, 9>;
  write_offset: [[stride(4)]] array<i32, 9>;
};
[[group(1), binding(0)]] var<uniform> params: TickTockUniforms;

fn equilibrium(velocity: vec2<f32>, rho: f32, direction: i32, usqr: f32) -> f32 {
  let e_dot_u = dot(e(direction), velocity);
  // internal fn pow(x, y) requires x cannot be negative
  return rho * w(direction) * (1.0 + 3.0 * e_dot_u + 4.5 * (e_dot_u * e_dot_u) - usqr);
}

// pull scheme
fn streaming_in(uv : vec2<i32>, direction : i32)->i32 {
    var target_uv : vec2<i32> = uv + vec2<i32>(e(fluid.inversed_direction[direction].x));  
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
    return latticeIndex(target_uv, direction);
}

[[stage(compute), workgroup_size(64, 4)]]
fn main([[builtin(global_invocation_id)]] GlobalInvocationID: vec3<u32>) {
    let uv = vec2<i32>(GlobalInvocationID.xy);
    if (uv.x >= field.lattice_size.x || uv.y >= field.lattice_size.y) {
      return;
    }
    var field_index : i32 = fieldIndex(uv);
    var info: LatticeInfo = lattice_info.data[field_index];
    if (isBoundaryCell(info.material) || isObstacleCell(info.material)) {
      textureStore(macro_info, vec2<i32>(uv), vec4<f32>(0.0, 0.0, 0.0, 0.0));
      return;
    }
    
    var f_i : array<f32, 9>;
    var velocity : vec2<f32> = vec2<f32>(0.0, 0.0);
    var rho : f32 = 0.0;
    for (var i : i32 = 0; i < 9; i = i + 1) {
      f_i[i] = aa_cell.data[field_index + params.read_offset[i] + soaOffset(i)];
      rho = rho + f_i[i];
      velocity = velocity + e(i) * f_i[i];
    }
    // reset lbm field sometimes cause Nan?
    // external force sometimes can cause Inf.
    if (isOutletCell(info.material)) {
      rho = 1.0;
    } else {
      rho = clamp(rho, 0.8, 1.2);
    }

    velocity = velocity / rho;
    // external forcing
    var F : array<f32, 9> = array<f32, 9>(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    // let force_x: f32 = 8.0 * 0.35 * 0.1 / (200.0 * 200.0);
    if (isAccelerateCell(info.material)) {
      if (info.block_iter > 0) {
        info.block_iter = info.block_iter - 1;
        if (info.block_iter == 0) {
          info.material = 1;
        }
      }
      lattice_info.data[field_index] = info;

      let force = vec2<f32>(info.vx, info.vy);
      // velocity.x = velocity.x + force_x * 0.5 / rho;
      velocity = force * 0.5 / rho;

      for (var i : i32 = 0; i < 9; i = i + 1) {
        F[i] = w(i) * 3.0 * dot(e(i), force);
      }
    }
   
    textureStore(macro_info, vec2<i32>(uv), vec4<f32>(velocity.x, velocity.y, rho, 1.0));

    let usqr = 1.5 * dot(velocity, velocity);
    for (var i : i32 = 0; i < 9; i = i + 1) {
      var temp_val: f32 = f_i[i] - fluid.omega * (f_i[i] - equilibrium(velocity, rho, i, usqr)) + F[i];
      if (temp_val > max_value(i) || isInf(temp_val)) {
        temp_val = max_value(i);
      } elseif (temp_val < 0.0) {
        temp_val = 0.0;
      }
      aa_cell.data[field_index + params.write_offset[i] + soaOffset(fluid.inversed_direction[i].x)] = temp_val;
    }
}
