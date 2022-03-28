
struct Particle3D {
    pos: vec4<f32>,
    // initial position, use to reset particle position
    pos_initial: vec4<f32>,
};

struct ParticleBuffer {
    particles: array<Particle3D>,
};