struct Particle {
   pos: vec4<f32>;
   old_pos: vec4<f32>;
  // 与之相连的4个粒子的索引，用于计算法线
   connect: vec4<i32>;
};

struct ParticlesBuffer {
    data: [[stride(48)]] array<Particle>;
};

fn is_movable_particle(particle: Particle) -> bool {
  if (particle.pos.w < 0.001) {
    return false;
  }
  return true;
}
