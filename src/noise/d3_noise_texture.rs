use crate::noise::{create_gradient_buf, create_permulation_buf};
use idroid::node::{BufferlessFullscreenNode, ComputeNode, ViewNode, ViewNodeBuilder};

pub struct D3NoiseTexture {
    pub tex: idroid::AnyTexture,
}

impl D3NoiseTexture {
    pub fn create(app_view: &idroid::AppView) -> Self {
        let device_features = app_view.device.features();
        // compressed texture format investigation: https://github.com/gpuweb/gpuweb/issues/144
        let tex_format = if device_features.contains(wgpu::Features::TEXTURE_COMPRESSION_ASTC_LDR) {
            // iOS，19% vulkan device
            wgpu::TextureFormat::Astc4x4RgbaUnormSrgb
        } else if device_features.contains(wgpu::Features::TEXTURE_COMPRESSION_ETC2) {
            // iOS，22% vulkan device
            wgpu::TextureFormat::Etc2RgbUnormSrgb
        } else if device_features.contains(wgpu::Features::TEXTURE_COMPRESSION_BC) {
            // macOS, 100% D3D12 device
            wgpu::TextureFormat::Bc1RgbaUnormSrgb
        } else {
            wgpu::TextureFormat::Rgba8Unorm
        };
        let tex = idroid::load_texture::empty(
            &app_view.device,
            wgpu::TextureFormat::Rgba8Unorm,
            wgpu::Extent3d { width: 64, height: 64, depth_or_array_layers: 64 },
            Some(wgpu::TextureViewDimension::D3),
            Some(wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING),
            None,
        );

        let dispatch_group_count = (8, 8, 8);

        let permulation_buf = create_permulation_buf(&app_view.device);
        let gradient_buf = create_gradient_buf(&app_view.device);
        let shader =
            idroid::shader::create_shader_module(&app_view.device, "noise/3d_noise_tex", None);
        let noise_node = ComputeNode::new(
            &app_view.device,
            dispatch_group_count,
            vec![],
            vec![&permulation_buf, &gradient_buf],
            vec![(&tex, Some(wgpu::StorageTextureAccess::WriteOnly))],
            &shader,
        );

        let mut encoder = app_view.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("3d noise encoder"),
        });
        noise_node.compute(&mut encoder);
        app_view.queue.submit(Some(encoder.finish()));

        Self { tex }
    }
}
