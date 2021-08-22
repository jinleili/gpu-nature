
struct LatticeInfo {
  material: i32;
   //  dynamic iter value, change material ultimately
  block_iter: i32;
  vx: f32;
  vy: f32;
};

[[block]]
struct StoreInfo {
    data: [[stride(16)]] array<LatticeInfo>;
};