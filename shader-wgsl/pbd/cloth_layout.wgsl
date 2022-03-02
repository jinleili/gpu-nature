#include "pbd/struct/particle.wgsl"
#include "pbd/struct/cloth_uniform.wgsl"

struct Constraint {
   rest_length: f32;
   lambda: f32;
   particle0: i32;
   particle1: i32;
};

struct ConstraintsBuffer {
    data: array<Constraint>;
};

// 这个粒子关联的所有约束
struct ParticleConstraints {
  list: array<i32, 3>;
};

struct ParticleConstraintsBuffer {
    data: array<ParticleConstraints>;
};

@group(0) @binding(0) var<uniform> cloth: ClothUniform;
@group(0) @binding(1) var<storage, read_write> particles: ParticlesBuffer;
@group(0) @binding(2) var<storage, read_write> constraints: ConstraintsBuffer;
@group(0) @binding(3) var<storage, read_write> reorder_constraints: ParticleConstraintsBuffer;

let EPSILON: f32 = 0.0000001;

fn is_movable_particle(particle: Particle) -> bool {
  if (particle.uv_mass.z < 0.001) {
    return false;
  }
  return true;
}
