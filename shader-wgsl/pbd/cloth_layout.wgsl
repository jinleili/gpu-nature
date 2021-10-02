#include "pbd/struct/particle.wgsl"
#include "pbd/struct/cloth_uniform.wgsl"

struct ConstraintObj {
   rest_length: f32;
   lambda: f32;
   particle0: i32;
   particle1: i32;
};
[[block]]
struct ConstraintObjs {
    data: [[stride(16)]] array<ConstraintObj>;
};

// 这个粒子关联的所有约束
struct ParticleConstraints {
  list: [[stride(4)]] array<i32, 8>;
};
[[block]]
struct Constraints {
    data: [[stride(32)]] array<ParticleConstraints>;
};

[[group(0), binding(0)]] var<uniform> cloth: ClothUniform;
[[group(0), binding(1)]] var<storage, read_write> particles: Particles;
[[group(0), binding(2)]] var<storage, read_write> constraints: ConstraintObjs;
[[group(0), binding(3)]] var<storage, read_write> particle_constraints: Constraints;
[[group(0), binding(4)]] var<storage, read_write> reorder_constraints: Constraints;

let EPSILON: f32 = 0.0000001;

fn is_movable_particle(particle: ParticleObj) -> bool {
  if (particle.uv_mass.z < 0.001) {
    return false;
  }
  return true;
}
