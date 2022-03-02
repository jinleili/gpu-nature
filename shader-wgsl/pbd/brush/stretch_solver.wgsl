#include "pbd/brush/struct/particle.wgsl"
#include "pbd/brush/struct/brush_uniform.wgsl"

struct Constraint {
   rest_length: f32;
   particle0: i32;
   particle1: i32;
};

struct ConstraintsBuffer {
    data: array<Constraint>;
};

// 这个粒子关联的所有 stretch 约束
struct ParticleConstraints {
    list: array<i32, 3>;
};

struct ParticleConstraintsBuffer {
    data: array<ParticleConstraints>;
};

@group(0) @binding(0) var<uniform> brush: BrushUniform;
@group(0) @binding(1) var<storage, read_write> particles: ParticlesBuffer;
@group(0) @binding(2) var<storage, read_write> constraints: ConstraintsBuffer;
@group(0) @binding(3) var<storage, read_write> reorder_constraints: ParticleConstraintsBuffer;

let EPSILON: f32 = 0.0000001;
#include "pbd/struct/dynamic_uniform.wgsl"
@group(1) @binding(0) var<uniform> dy_uniform: DynamicUniform;

// mesh coloring 分组之后，最后一组往往约束组数量很小（64*64粒子，最小约束组长度是 94）
@stage(compute) @workgroup_size(32, 1)
fn cs_main(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {  
    var field_index = i32(global_invocation_id.x);
    if (field_index >= dy_uniform.group_len) {
        return;
    }
    field_index = field_index + dy_uniform.offset;

    var particle_constraints = reorder_constraints.data[field_index];
    var particle0_index = 0;
    var particle: Particle;
    var invert_mass0 = -1.0;
    // a~
    // new_compliance 直接在 uniform 里计算好
    // float new_compliance = compliance / (dt * dt);
    // 遍历所有约束
    for (var i = 0; i < 1; i = i + 1) {
        let constraint_index = particle_constraints.list[i];
        if (constraint_index < 0) {
            continue;
        }
        // storage buffer item 里的字段如果有数组，那 item 只能使用 var 绑定，否则后面读取 item 字段时会报错：
        // The expression [73] may only be indexed by a constant
        var constraint = constraints.data[constraint_index];
        if (invert_mass0 < 0.0) {
            particle0_index = constraint.particle0;
            particle = particles.data[particle0_index];
            invert_mass0 = particle.pos.w;
        }

        var particle1 = particles.data[constraint.particle1];
        let invert_mass1 = particle1.pos.w;
        let sum_mass = invert_mass0 + invert_mass1;
        if (sum_mass < 0.01) {
            continue;
        }
        let p0_minus_p1 = particle.pos - particle1.pos;
        let dis = length(p0_minus_p1.xyz);
        // Cj(x)
        let distance = dis - constraint.rest_length;

        var correction_vector: vec4<f32>;
        // eq.18
        let dlambda = -distance / (sum_mass + brush.compliance);
        // eq.17
        correction_vector = dlambda * p0_minus_p1 / (dis + EPSILON);

        // 更新位置
        if (is_movable_particle(particle)) {
            particle.pos = particle.pos + invert_mass0 * correction_vector;
        }
        if (is_movable_particle(particle1)) {
            particle1.pos = particle1.pos + (-invert_mass1) * correction_vector;
            particles.data[constraint.particle1] = particle1;
        }
    }

    // particle0 是约束组通用的，最后一次性写入就可
    // invert_mass0 > 0.0 用于避免 particle 被错误地覆盖
    if (invert_mass0 > 0.0 && is_movable_particle(particle)) {
        particles.data[particle0_index] = particle;
    }
}