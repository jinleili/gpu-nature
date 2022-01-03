#include "func/color_space_convert.wgsl"


struct MVPMatUniform {
    mv: mat4x4<f32>;
    proj: mat4x4<f32>;
    mvp: mat4x4<f32>;
    normal: mat4x4<f32>;
};

[[group(0), binding(0)]] var<uniform> mvp_mat: MVPMatUniform;


struct Uniforms {
    light_x: f32;
    light_y: f32;
    light_z: f32;
    d: f32;
};
[[group(0), binding(1)]] var<uniform> params: Uniforms;
// wavelength
let LAMBDA_MIN: f32 = 400.0;
let LAMBDA_MAX: f32 = 600.0;

fn is_valid_wavelength(lambda: f32) -> bool {
    if (lambda < LAMBDA_MIN || lambda > LAMBDA_MAX) {
        return false;
    } else {
        return true;
    }
}

fn rainbow(t: f32) -> vec3<f32> {
    let t = clamp(t, 0., 1.);
    // var color: vec3<f32>;
    // if (t >= 0.75) {
    //     color = vec3<f32>(1., 1. - 4. * (t - 0.75), 0.);
    // } else if (t >= 0.5) {
    //     color = vec3<f32>(1. - 4. * (t - 0.5), 1., 0.);
    // } else if (t >= 0.25) {
    //     color = vec3<f32>(0., 1., 1. - 4. * (t - 0.25));
    // } else {
    //     color = vec3<f32>(0., 4. * t,  1.);
    // }
    // return color;
    return hsv2rgb(t, 0.95, 0.95);
}


struct VertexOutput {
    [[location(0)]] color: vec3<f32>;
    [[builtin(position)]] position: vec4<f32>;
};

[[stage(vertex)]]
fn vs_main(
    [[location(0)]] pos: vec3<f32>,
    [[location(1)]] tangent: vec4<f32>,
) -> VertexOutput {
    let ec_pos = (mvp_mat.mv * vec4<f32>(pos, 1.0)).xyz;
    let transf_tangent = (mvp_mat.normal * tangent).xyz;

    let to_light = normalize(vec3<f32>(params.light_x, params.light_y, params.light_z) - ec_pos);
    let to_eye = normalize(vec3<f32>(0.0) - ec_pos);

    let sum = dot(to_light + to_eye, normalize(transf_tangent));
    let delta = params.d *  abs(sum);
    let m_min = i32(floor(delta / LAMBDA_MAX));
    let m_max = i32(ceil(delta / LAMBDA_MIN));
    var out_color = vec3<f32>(0.75);
    // var frag_color =  vec4<f32>(hsv2rgb(in.transf_tangent.y, in.transf_tangent.y, 1.0), 1.0);
    if (m_min > 0) {
        var color: vec3<f32> = vec3<f32>(0.0);
        var count: i32 = 0;
        for (var m: i32 = m_min; m <= m_max; m = m + 1) {
            let lambda = delta / f32(m);
            if (is_valid_wavelength(lambda)) {
                color = color + rainbow((lambda - LAMBDA_MIN) / (LAMBDA_MAX - LAMBDA_MIN));
                count = count + 1;
            }
        }
        if (count > 0) {
            out_color = out_color * 0.5 + (color / f32(count)) * 0.5;
        }
    } 

    var output: VertexOutput;
    output.position = mvp_mat.mvp * vec4<f32>(pos, 1.0);
    output.color = out_color;
    return output;
}


[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}