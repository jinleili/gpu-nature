use std::collections::HashMap;
use std::error::Error;
use std::io::prelude::*;

pub struct ShaderMapGenerator {
    shaders: HashMap<String, String>,
}

impl ShaderMapGenerator {
    pub fn new() -> Self {
        Self { shaders: HashMap::new() }
    }
    pub fn insert(&mut self, key: String, val: String) {
        self.shaders.insert(key, val);
    }

    pub fn generate_code(&self) -> Result<(), Box<dyn Error>> {
        let mut code = String::from(
            r#"
use std::collections::HashMap;
lazy_static::lazy_static! {
    pub static ref SHARDERMAP: HashMap<String, String> = {
        let mut m = HashMap::new();
        "#,
        );
        for (k, v) in self.shaders.iter() {
            let mut s = String::from("        m.insert(\"");
            s += k;
            s += "\".to_string(), \"";
            s += v;
            s += "\".to_string());";
            code.push_str(&s);
        }
        code.push_str(
            r#"
        m
    };
}
        "#,
        );

        let base_dir = env!("CARGO_MANIFEST_DIR");
        let out_path = "src/web/generated_shader_map.rs".to_string();

        let path = std::path::Path::new(&base_dir).join(&out_path);
        let mut f = if let Ok(f) = std::fs::File::create(&path) {
            f
        } else {
            std::fs::File::create(&path).unwrap()
        };
        f.write_all(code.as_bytes())?;

        Ok(())
    }
}
