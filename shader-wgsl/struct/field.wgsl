
struct FieldUniform {
  // field lattice number
  lattice_size: vec4<i32>;
  lattice_pixel_size: vec4<f32>;
  // canvas pixel number
  canvas_size: vec4<i32>;
  normalized_space_size: vec4<f32>;
  // the value corresponding to one pixel in the normalized coordinate
  // space
  pixel_distance: vec4<f32>;
  // 0: pixel speed, field player used 
  // 1: lbm lattice speed, fluid player used. Its value is usually no greater than 0.2
  speed_ty: i32;
};


struct FieldBuffer {
    data: [[stride(16)]] array<vec4<f32>>;
};
