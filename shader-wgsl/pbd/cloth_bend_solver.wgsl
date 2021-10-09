#include "pbd/struct/particle.wgsl"
#include "pbd/struct/dynamic_uniform.wgsl"
#include "pbd/cloth_layout.wgsl"

struct BendConstraint {
   p0: i32;
   p1: i32;
   p2: i32;
   p3: i32;
};
[[block]]
struct BendConstraintBuf {
    data: [[stride(16)]] array<BendConstraint>;
};

[[block]]
struct BendConstraintGoupBuf {
    group: [[stride(12)]] array<array<i32, 3>>;
};

[[group(0), binding(0)]] var<uniform> cloth: ClothUniform;
[[group(0), binding(1)]] var<storage, read_write> particles: ParticlesBuffer;
[[group(0), binding(2)]] var<storage, read_write> constraints: BendConstraintBuf;
[[group(0), binding(3)]] var<storage, read_write> reorder_constraints: BendConstraintGoupBuf;

[[group(1), binding(0)]] var<uniform> dy_uniform: DynamicUniform;


fn is_movable_particle(particle: Particle) -> bool {
  if (particle.uv_mass.z < 0.001) {
    return false;
  }
  return true;
}
// 初始双面角 ϕ0
let phi0 = 3.1415926535;
[[stage(compute), workgroup_size(32, 1)]]
fn main([[builtin(global_invocation_id)]] global_invocation_id: vec3<u32>) {  
    var field_index = i32(global_invocation_id.x);
    if (field_index >= dy_uniform.group_len) {
        return;
    }
    field_index = field_index + dy_uniform.offset;
    var group: array<i32, 3> = reorder_constraints.group[field_index];
    for (var i = 0; i < 1; i = i + 1) {
        let constraint_index = group[i];
        if (constraint_index == -1) {
            continue;
        }

        var bend: BendConstraint = constraints.data[constraint_index];
        // 弯曲约束 C = acos(d) -ϕ0, d = n1.n2
        let particle0 = particles.data[bend.p0];
        let particle1 = particles.data[bend.p1];
        var particle2 = particles.data[bend.p2];
        var particle3 = particles.data[bend.p3];
        let p0 = vec3<f32>(0.0);
        let p1 = particle1.pos.xyz - particle0.pos.xyz;
        let p2 = particle2.pos.xyz - particle0.pos.xyz;
        let p3 = particle3.pos.xyz - particle0.pos.xyz;

        let p1p2 = cross(p1, p2);
        let n0 = normalize(p1p2);
        let p1p3 = cross(p1, p3);
        let n1 = normalize(p1p3);
        let d = dot(n0, n1);
        if (isNan(d)) {
            return;
        }

        // eq. 25, 25
        let q2 = (cross(p1, n1) + cross(n0, p1) * d) / length(p1p2);
        let q3 = (cross(p1, n0) + cross(n1, p1) * d) / length(p1p3);
        let q1 = -(cross(p2, n1) + cross(n0, p2) * d) / length(p1p2) - (cross(p3, n0) + cross(n1, p3) * d) / length(p1p3);
        let q0 = -q1 - q2 - q3;

        // float sum_wj = particle0.uv_mass.z + particle1.uv_mass.z + particle2.uv_mass.z + particle3.uv_mass.z;
        // float sum_qj = pow(length(q0), 2.0) + pow(length(q1), 2.0) + pow(length(q2), 2.0) + pow(length(q2), 2.0);
        let sum_wq = particle0.uv_mass.z * pow(length(q0), 2.0) + particle1.uv_mass.z * pow(length(q1), 2.0) +
                    particle2.uv_mass.z * pow(length(q2), 2.0) + particle3.uv_mass.z * pow(length(q3), 2.0);

        // θ=acos(v1⋅v2/||v1||||v2||)
        // float phi = acos(d / (length(p1p2) * length(p1p3)));
        let pre_val = sqrt(1.0 - d * d) * (acos(d) - phi0);
        let dp2 = (particle2.uv_mass.z * pre_val) / sum_wq * (-q2);
        let dp3 = (particle3.uv_mass.z * pre_val) / sum_wq * (-q3);
        if (isNan(sum_wq) || isNan(dp2.x) || isNan(dp3.x)) {
            return;
        }
        if (is_movable_particle(particle2)) {
            // 公式 29
            particle2.pos = vec4<f32>(particle2.pos.xyz + dp2, particle2.pos.w);
            particles.data[bend.p2] = particle2;
        }
        if (is_movable_particle(particle3)) {
            particle3.pos = vec4<f32>(particle3.pos.xyz + dp3, particle3.pos.w);
            particles.data[bend.p3] = particle3;
        }
    }
}