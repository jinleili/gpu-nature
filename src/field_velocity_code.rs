use crate::FieldAnimationType;

pub fn get_velocity_code_segment(ty: FieldAnimationType) -> &'static str {
    match ty {
        FieldAnimationType::Basic => {
            r#"
    let new_y = f32(p.y) - f32(field.lattice_size.y) / 2.0;
    var v: vec2<f32> = vec2<f32>(0.0, 0.0);
    v.x = 0.1 * new_y;
    v.y = -0.2 * new_y;
    return v * 0.5;
    "#
        }
        FieldAnimationType::JuliaSet => {
            r#"
    var c: vec2<f32> = vec2<f32>(p) / (vec2<f32>(field.lattice_size) / 2.0) - vec2<f32>(1.0, 1.0);
    c = c * field.normalized_space_size;
    let z = vec2<f32>(0.4, 0.5);
    for (var i: i32 = 0; i < 8; i = i + 1) {
        c = vec2<f32>(c.x * c.x - c.y * c.y, c.y * c.x + c.x * c.y);
        c = c + z;
    }
    return c * 4.0;
    "#
        }
        FieldAnimationType::Spirl => {
            r#"
    var c: vec2<f32> = vec2<f32>(p) / (vec2<f32>(field.lattice_size) / 2.0) - vec2<f32>(1.0, 1.0);
    let r = length(c);
    let theta = atan2(c.y, c.x);
    var v: vec2<f32> = vec2<f32>(c.y, -c.x) / r;
    let t = sqrt(r * 10.0) + theta + 0.1; // + frame * 0.02;
    v = v * sin(t);
    v = v * length(v) * 10.0;
    return v + c * 0.2;
    "#
        }
        _ => "",
    }
}
