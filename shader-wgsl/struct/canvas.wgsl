struct Pixel {
    alpha: f32;
    velocity_x: f32;
    velocity_y: f32;
};

struct CanvasBuffer {
    pixels: [[stride(12)]] array<Pixel>;
};