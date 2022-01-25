#include "bufferless.vs.wgsl"

#include "struct/field.wgsl"
#include "struct/particle.wgsl"
#include "struct/canvas.wgsl"

@group(0) @binding(0) var<uniform> field: FieldUniform;
@group(0) @binding(1) var<uniform> particle_uniform: ParticleUniform;
@group(0) @binding(2) var<storage, read_write> canvas: CanvasBuffer;

#include "func/color_space_convert.wgsl"

let PI: f32 = 3.1415926535;

@stage(fragment) 
fn fs_main(@builtin(position) coord: vec4<f32>) -> @location(0) vec4<f32> {
    let pixel_coord = min(vec2<i32>(floor(coord.xy)), field.canvas_size.xy - 1);
    let p_index = pixel_coord.x + pixel_coord.y * field.canvas_size.x;
    var p: Pixel = canvas.pixels[p_index];

    var frag_color: vec4<f32>;
    if (p.alpha > 0.001) {
        // frag_color = vec4<f32>(particle_uniform.color.rgb, 0.0);
        if (particle_uniform.color_ty == 2) {
            // speed as color
            let velocity = abs(p.velocity_x) + abs(p.velocity_y);
            var speed: f32;
            if (field.speed_ty == 0) {
                speed =  velocity / max((f32(field.canvas_size.x) / particle_uniform.speed_factor), (f32(field.canvas_size.y) / particle_uniform.speed_factor));
            } else {
                speed =  min(velocity / 0.25, 1.15);
            }
            frag_color = vec4<f32>(hsv2rgb(0.05 + speed * 0.75, 0.9, 1.0), p.alpha);
        } else if (particle_uniform.color_ty == 1) {
            // moving angle as color
            let angle = atan2(p.velocity_y, p.velocity_x) / (2.0 * PI);
            frag_color = vec4<f32>(hsv2rgb(angle + 0.5, 0.9, 1.0), p.alpha);
        } else {
            frag_color = vec4<f32>(particle_uniform.color.rgb, p.alpha);
        }

        // fade out trajectory
        if (p.alpha >= 0.2) {
            p.alpha = p.alpha * particle_uniform.fade_out_factor;
        } else {
            p.alpha = p.alpha * 0.5;
        }
        // p.alpha = 0.0;
        canvas.pixels[p_index] = p;
    } else {
        frag_color = vec4<f32>(particle_uniform.color.rgb, 0.0);
    }
    return frag_color;
}

