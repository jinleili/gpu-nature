#include "lbm/struct/lbm_uniform.wgsl"
#include "struct/field.wgsl"
#include "struct/particle.wgsl"
#include "struct/canvas.wgsl"

[[group(0), binding(0)]] var<uniform> fluid: LbmUniform;
[[group(0), binding(1)]] var<uniform> field: FieldUniform;
[[group(0), binding(2)]] var<uniform> particle_uniform: ParticleUniform;
[[group(0), binding(3)]] var<storage, read_write> pb: ParticleBuffer;
[[group(0), binding(4)]] var<storage, read_write> canvas: CanvasBuffer;
[[group(0), binding(5)]] var fb: texture_2d<f32>;

fn isPoiseuilleFlow() -> bool { return fluid.fluid_ty == 0; }

fn src_3f(u: i32, v: i32) -> vec3<f32> {
  let new_u = clamp(u, 0, field.lattice_size.x - 1);
  let new_v = clamp(v, 0, field.lattice_size.y - 1);
//   let index = new_v * field.lattice_size.x + new_u;
//   return fb.data[index].xyz;
  return vec3<f32>(textureLoad(fb, vec2<i32>(new_u, new_v), 0).xyz);
}
#include "func/bilinear_interpolate_3f.wgsl"

fn field_index(uv: vec2<i32>) -> i32 {
   return uv.x + (uv.y * field.lattice_size.x);
}

fn particle_index(uv: vec2<i32>) -> i32 {
   return uv.x + (uv.y * particle_uniform.num.x);
}

fn update_canvas(particle: TrajectoryParticle, velocity: vec2<f32>) {
    let speed = abs(velocity.x) + abs(velocity.y);
    // keep obstacle area blank
    if ((isPoiseuilleFlow() == false && speed < 0.035) || (isPoiseuilleFlow() && speed < 0.015)) {
        return;
    }
    // first, need calculate out pixel coordinate
    let pixel_coords = vec2<i32>(particle.pos);
    let px = pixel_coords.x - particle_uniform.point_size / 2;
    let py = pixel_coords.y - particle_uniform.point_size / 2;
    // then, update values by scope
    let pixel = Pixel(particle.fade, velocity.x, velocity.y);
    for (var x: i32 = 0; x < particle_uniform.point_size; x = x + 1) {
        for (var y: i32 = 0; y < particle_uniform.point_size; y = y + 1) {
            let coords = vec2<i32>(px + x, py + y);
            if (coords.x >= 0 && coords.x < field.canvas_size.x 
                && coords.y >= 0 && coords.y < field.canvas_size.y) {
                canvas.pixels[coords.x + field.canvas_size.x * coords.y] = pixel;
            }
        }
    }
}

[[stage(compute), workgroup_size(16, 16)]]
fn main([[builtin(global_invocation_id)]] GlobalInvocationID: vec3<u32>) {
    let uv = vec2<i32>(GlobalInvocationID.xy);
    if (uv.x >= particle_uniform.num.x || uv.y >= particle_uniform.num.y) {
        return;
    }
    let p_index: i32 = particle_index(uv);
    var particle: TrajectoryParticle = pb.particles[p_index];
     // Calculate which lattice this particle is located
    let ij = (particle.pos / field.lattice_pixel_size.xy) - 0.5;
    let field_info = bilinear_interpolate_3f(ij);
    if (abs(field_info.x) < 0.0001 && abs(field_info.y) < 0.0001) {
        particle.fade = 0.0;
    } else {
        // fade in effect
        if (particle.fade < 1.0) {
            if (particle.fade < 0.9) {
                particle.fade = particle.fade + 0.1;
            } else {
                particle.fade = 1.0;
            }
        }   
        particle.pos = particle.pos + (field_info.xy * particle_uniform.speed_factor);
    }
    
    let pixel_coords = vec2<i32>(particle.pos);
    if (pixel_coords.x < 0 || pixel_coords.x > field.canvas_size.x || pixel_coords.y < 0 || pixel_coords.y > field.canvas_size.y ) {
        particle.pos = particle.pos_initial;
        particle.fade = 0.0;
    }
   
    pb.particles[p_index] = particle;
}