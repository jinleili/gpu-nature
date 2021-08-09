struct Pixel {
    alpha: f32;
    velocity_x: f32;
    velocity_y: f32;
};
[[block]]
struct CanvasBuffer {
    pixels: [[stride(12)]] array<Pixel>;
};