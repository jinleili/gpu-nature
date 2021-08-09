[[block]]
struct ParticleUniform {
    num: i32;
    point_size: i32;
    fade_out_factor: f32;
    // pixels moved per unit speed 
    speed_factor: f32;
};

struct TrajectoryParticle {
    pos: vec2<f32>;
    // initial position, use to reset particle position
    pos_initial: vec2<f32>;
    life_time: f32;
    // alpha value:[1.0, 0.0]
    fade: f32;
};
[[block]]
struct ParticleBuffer {
    particles: [[stride(24)]] array<TrajectoryParticle>;
};