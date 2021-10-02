#include "pbd/struct/particle.wgsl"
#include "pbd/struct/collision.wgsl"
#include "pbd/struct/cloth_uniform.wgsl"

struct TriangleObj {
  p0: i32;
  p1: i32;
  p2: i32;
};
[[block]]
struct TriangleObjBuf {
  data: [[stride(12)]] array<TriangleObj>;
};


struct ConstraintObj {
   rest_length: f32;
   lambda: f32;
   particle0: i32;
   particle1: i32;
};
[[block]]
struct ConstraintObjBuf {
    data: [[stride(16)]] array<ConstraintObj>;
};

struct BendingConstraintObj {
   v: i32;
   b0: i32;
   b1: i32;
  // h 0 is the rest length (rest radius of curvature)
   h0: f32;
};
[[block]]
struct BendingConstraintObjBuf {
    data: [[stride(16)]] array<BendingConstraintObj>;
};

// 这个粒子关联的所有约束
struct ParticleConstraints {
  // 拉伸约束 stretches 的索引
  stretches: array<i32, 4>;
  bendings: array<i32, 4>;
};
[[block]]
struct ParticleConstraintsBuf {
    data: [[stride(32)]] array<ParticleConstraints>;
};

struct BinObj {
  // 第 0 位表示帧索引， 第 1 位表示已归入hash网格的顶点数
 list: array<i32, 16>;
};
[[block]]
struct BinObjBuf {
    data: [[stride(64)]] array<BinObj>;
};

[[block]]
struct ColoringBuf {
   data: [[stride(4)]] array<i32>;
};

[[block]]
struct BinUniform {
  // bin hash 容器数
   bin_num: vec4<i32>;
  // 容器各轴向上最大的索引数
   bin_max_index: vec4<i32>;
   bin_size: vec4<f32>;
  // 转换到 【0～n]坐标空间需要的偏移
   pos_offset: vec4<f32>;
   max_bin_count: i32;
};

[[block]]
struct FrameUniform {
    frame_index: i32;
};

[[group(0), binding(0)]] var<uniform> cloth: ClothUniform;
[[group(0), binding(1)]] var<uniform> bin: BinUniform;
[[group(0), binding(2)]] var<uniform> frame: FrameUniform;

[[group(0), binding(3)]] var<storage, read_write> particles: ParticleObjBuf;
[[group(0), binding(4)]] var<storage, read_write> bins: BinObjBuf;
[[group(0), binding(5)]] var<storage, read_write> particle_constraints: ParticleConstraintsBuf;
[[group(0), binding(6)]] var<storage, read_write> stretches: ConstraintObjBuf;
[[group(0), binding(7)]] var<storage, read_write> bendings: BendingConstraintObjBuf;
// 网络着色法粒子分组
[[group(0), binding(8)]] var<storage, read_write> mesh_coloring: ColoringBuf;
[[group(0), binding(9)]] var<storage, read_write> triangles: TriangleObjBuf;
[[group(0), binding(10)]] var<storage, read_write> collisions: CollisionObjBuf;

struct DebugObj {
   val0: vec4<i32>;
   val1: vec4<i32>;
   val2: vec4<f32>;
   val3: vec4<f32>;
};
[[block]]
struct DebugObjBuf {
  data: [[stride(64)]] array<DebugObj>;
};
// [[group(0), binding(11)]] var<storage> debugs: DebugObjBuf;

let EPSILON: f32 = 0.000001;
let cloth_thickness: f32 = 0.0025;

fn linearize_bin_index(pos: vec3<f32>) -> i32 {
  let bin_pos: vec3<i32> = vec3<i32>((pos + bin.pos_offset.xyz) / bin.bin_size.xyz);
  if (bin_pos.x < 0 || bin_pos.x > bin.bin_max_index.x || bin_pos.y < 0 || bin_pos.y > bin.bin_max_index.y || bin_pos.z < 0 ||
      bin_pos.z > bin.bin_max_index.z) {
    return -1;
  }
  return (bin.bin_num.x * bin.bin_num.y) * bin_pos.z + bin.bin_num.x * bin_pos.y + bin_pos.x;
}

fn linearize_bin_index2(x: i32, y: i32, z: i32) -> i32 {
   return (bin.bin_num.x * bin.bin_num.y) * z + bin.bin_num.x * y + x; 
}

fn is_movable_particle(particle: ParticleObj) -> bool {
  if (particle.uv_mass.z < 0.001) {
    return false;
  }
  return true;
}