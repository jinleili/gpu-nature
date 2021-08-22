use std::error::Error;
use std::fs::read_to_string;
use std::io::prelude::*;
use std::path::PathBuf;

mod build_for_wasm;
use build_for_wasm::ShaderMapGenerator;

// build.rs 配置：https://blog.csdn.net/weixin_33910434/article/details/87943334
fn main() -> Result<(), Box<dyn Error>> {
    let mut need_generate_shader = false;
    for (key, value) in std::env::vars() {
        if key == "PREPROCESS_SHADER" && value == "true" {
            need_generate_shader = true;
        }
    }
    if !need_generate_shader {
        return Ok(());
    }

    let shader_files = vec![
        "field_setting",
        "trajectory_update",
        "present",
        "clear_color",
        "lbm/collide_stream",
        "lbm/init",
        "lbm/boundary",
        "lbm/particle_update",
        "lbm/curl_update",
        "lbm/present",
        "lbm/trajectory_present",
    ];
    let mut map_generator = ShaderMapGenerator::new();

    // 创建目录
    std::fs::create_dir_all("shader-preprocessed-wgsl")?;
    for name in shader_files {
        let _ = regenerate_shader(name, &mut map_generator);
    }

    let _ = map_generator.generate_code();

    Ok(())
}

fn regenerate_shader(
    shader_name: &str, map_generator: &mut ShaderMapGenerator,
) -> Result<(), Box<dyn Error>> {
    let base_dir = env!("CARGO_MANIFEST_DIR");
    let path = PathBuf::from(&base_dir).join("shader-wgsl").join(format!("{}.wgsl", shader_name));
    let mut out_path = "shader-preprocessed-wgsl/".to_string();
    out_path += &format!("{}.wgsl", shader_name.replace("/", "_"));

    let code = match read_to_string(&path) {
        Ok(code) => code,
        Err(e) => {
            panic!("Unable to read {:?}: {:?}", path, e)
        }
    };

    let mut shader_source = String::new();
    parse_shader_source(&code, &mut shader_source, &base_dir);
    map_generator.insert(shader_name.replace("/", "_"), shader_source.clone());

    let mut f = std::fs::File::create(&std::path::Path::new(&base_dir).join(&out_path))?;
    f.write_all(shader_source.as_bytes())?;

    Ok(())
}

fn parse_shader_source(source: &str, output: &mut String, base_path: &str) {
    let include: &str = "#include ";
    for line in source.lines() {
        if line.starts_with(include) {
            let imports = line[include.len()..].split(',');
            // For each import, get the source, and recurse.
            for import in imports {
                if let Some(include) = get_shader_funcs(import, base_path) {
                    parse_shader_source(&include, output, base_path);
                } else {
                    println!("shader parse error -------");
                    println!("can't find shader functions: {}", import);
                    println!("--------------------------");
                }
            }
        } else {
            // 移除注释
            let need_delete = match line.find("//") {
                Some(_) => {
                    let segments: Vec<&str> = line.split("//").collect();
                    segments.len() > 1 && segments.first().unwrap().trim().is_empty()
                }
                None => false,
            };
            if !need_delete {
                output.push_str(line);
                output.push_str("\n");
            }
        }
    }
}

fn get_shader_funcs(key: &str, base_path: &str) -> Option<String> {
    let path = PathBuf::from(base_path).join("shader-wgsl").join(key.replace('"', ""));
    let shader = match read_to_string(&path) {
        Ok(code) => code,
        Err(e) => panic!("Unable to read {:?}: {:?}", path, e),
    };
    Some(shader)
}
