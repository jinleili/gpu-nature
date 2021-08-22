use super::generated_shader_map::*;
use std::{borrow::Cow, collections::HashMap, fs::read_to_string, path::PathBuf};
use wgpu::{ShaderFlags, ShaderModule, ShaderModuleDescriptor, ShaderSource};

const SHADER_SEGMENT: &str = "#insert_code_segment";

#[allow(dead_code)]
pub fn create_shader_module(
    device: &wgpu::Device, shader_name: &'static str, label: Option<&str>,
) -> ShaderModule {
    insert_code_then_create(device, shader_name, None, label)
}

#[allow(dead_code)]
pub fn insert_code_then_create(
    device: &wgpu::Device, shader_name: &'static str, code_segment: Option<&str>,
    label: Option<&str>,
) -> ShaderModule {
    let flags = ShaderFlags::VALIDATION;
    let shader_name = shader_name.replace("/", "_");

    let mut shader_source = String::from("");
    if let Some(code) = SHARDERMAP.get(&shader_name) {
        shader_source = code.to_string();
    } else {
        panic!("Unable to get {:?}", &shader_name);
    }

    let final_source = if let Some(segment) = code_segment {
        let mut output = String::new();
        for line in shader_source.lines() {
            if line.contains(SHADER_SEGMENT) {
                output.push_str(segment);
            } else {
                output.push_str(line);
            }
            output.push_str("\n ");
        }
        output
    } else {
        shader_source
    };

    device.create_shader_module(&ShaderModuleDescriptor {
        label,
        source: ShaderSource::Wgsl(Cow::Borrowed(&final_source)),
        flags,
    })
}
