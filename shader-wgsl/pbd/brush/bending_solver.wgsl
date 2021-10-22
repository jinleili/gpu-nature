#include "pbd/brush/struct/particle.wgsl"
#include "pbd/brush/struct/brush_uniform.wgsl"

struct BendingConstraint {
    v: i32;
    b0: i32;
    b1: i32;
    // h 0 is the rest length (rest radius of curvature)
    h0: f32;
};

[[block]]
struct BendingConstraintsBuffer {
    data: [[stride(16)]] array<BendingConstraint>;
};

// 这个粒子关联的所有弯曲约束
struct BendingConstraintsGroup {
    list: [[stride(4)]] array<i32, 2>;
};
[[block]]
struct BendingConstraintsGroupBuffer {
    data: [[stride(8)]] array<BendingConstraintsGroup>;
};

[[group(0), binding(0)]] var<uniform> brush: BrushUniform;
[[group(0), binding(1)]] var<storage, read_write> particles: ParticlesBuffer;
[[group(0), binding(2)]] var<storage, read_write> constraints: BendingConstraintsBuffer;
[[group(0), binding(3)]] var<storage, read_write> reorder_constraints: BendingConstraintsGroupBuffer;

[[block]]
struct DynamicUniform {
    offset: i32;
    max_num_x: i32;
    // 当前 mesh coloring 分组的数据长度
    group_len: i32;
    // 迭代計數的倒數
    invert_iter: f32;
};
[[group(1), binding(0)]] var<uniform> dy_uniform: DynamicUniform;


[[stage(compute), workgroup_size(32, 1)]]
fn main([[builtin(global_invocation_id)]] global_invocation_id: vec3<u32>) {  
    var field_index = i32(global_invocation_id.x);
    if (field_index >= dy_uniform.group_len) {
        return;
    }
    field_index = field_index + dy_uniform.offset;
    
    var group: BendingConstraintsGroup = reorder_constraints.data[field_index];
    for (var i = 0; i < 3; i = i + 1) {
        let constraint_index = group.list[i];
        if (constraint_index < 0) {
            continue;
        }

        let bending: BendingConstraint = constraints.data[constraint_index];
        // 弯曲约束 C = acos(d) -ϕ0, d = n1.n2
        var v: Particle = particles.data[bending.v];
        var b0: Particle = particles.data[bending.b0];
        var b1: Particle = particles.data[bending.b1];

        // eq. 3
        let c: vec3<f32> = (b0.pos.xyz + b1.pos.xyz + v.pos.xyz) * 0.33333333;
        // eq. 8
        let w = b0.pos.w + b1.pos.w + 2.0 * v.pos.w;
        let v_minus_c = v.pos.xyz - c;
        let v_minus_c_len = length(v_minus_c);
        // 公式 6
        let k = 1.0 - pow(1.0 - 0.15, dy_uniform.invert_iter);
        // float k = 0.0;
        // 不等式約束方程 5
        // 弯曲度大于静态值才执行位置修正（论文里的表述反了）
        let c_triangle = v_minus_c_len - (k + bending.h0);
        if (c_triangle <= 0.0) {
            return;
        }
        // 方程 9a, 9b, 9c
        let f = v_minus_c * (1.0 - (k + bending.h0) / v_minus_c_len);

        if (is_movable_particle(v)) {
            v.pos = vec4<f32>(v.pos.xyz + (-4.0 * v.pos.w) / w * f, 0.0);
            particles.data[bending.v] = v;
        }
        if (is_movable_particle(b0)) {
            b0.pos = vec4<f32>(b0.pos.xyz + (2.0 * b0.pos.w) / w * f, 0.0);
            particles.data[bending.b0] = b0;
        }
        if (is_movable_particle(b1)) {
            b1.pos = vec4<f32>(b1.pos.xyz + (2.0 * b1.pos.w) / w * f, 0.0);
            particles.data[bending.b1] = b1;
        }
    }
}
