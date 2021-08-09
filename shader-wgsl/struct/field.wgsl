[[block]]
struct FieldUniform {
  // field lattice number
  lattice_size: vec3<i32>;
  lattice_pixel_size: vec3<f32>;
  // canvas pixel number
  canvas_size: vec3<i32>;
  normalized_space_size: vec3<f32>;
  // the value corresponding to one pixel in the normalized coordinate
  // space
  pixel_distance: vec3<f32>;
  // 0: pixel speed, field player used 
  // 1: lbm lattice speed, fluid player used. Its value is usually no greater than 0.2
  speed_ty: i32;
};

[[block]]
struct FieldBuffer {
    data: [[stride(16)]] array<vec4<f32>>;
};
