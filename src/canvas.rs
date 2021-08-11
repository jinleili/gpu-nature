use crate::Diffraction;
use idroid::node::BufferlessFullscreenNode;
use idroid::{math::TouchPoint, SurfaceView};
use uni_view::{AppView, GPUContext};
use crate::noise::*;
pub struct Canvas {
    pub app_view: AppView,
    pub nature_node: Diffraction,
    pub noise_display: BufferlessFullscreenNode,
}

#[allow(dead_code)]
impl Canvas {
    pub fn new(app_view: AppView) -> Self {
        let nature_node = Diffraction::new(&app_view);
        let noise_shader =
            idroid::shader::create_shader_module(&app_view.device, "perlin_noise", None);
        let permulation_buf = create_permulation_buf(&app_view.device);
        let gradient_buf = create_gradient_buf(&app_view.device);
        let noise_display = BufferlessFullscreenNode::new(
            &app_view.device,
            app_view.sc_desc.format,
            vec![],
            vec![&permulation_buf, &gradient_buf],
            vec![],
            vec![],
            Some(vec![wgpu::ShaderStages::FRAGMENT, wgpu::ShaderStages::FRAGMENT]),
            &noise_shader,
        );

        Self { app_view, nature_node, noise_display }
    }
}

impl SurfaceView for Canvas {
    fn resize(&mut self) {}
    fn pintch_start(&mut self, _location: TouchPoint, _scale: f32) {}
    fn pintch_changed(&mut self, _location: TouchPoint, _scale: f32) {}
    fn touch_start(&mut self, _point: TouchPoint) {}
    fn touch_moved(&mut self, _point: TouchPoint) {}
    fn touch_end(&mut self, _point: TouchPoint) {}

    fn enter_frame(&mut self) {
        // self.nature_node.enter_frame(&mut self.app_view);

        let mut encoder =
            self.app_view.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("diffraction encoder"),
            });
        let frame = match self.app_view.swap_chain.get_current_frame() {
            Ok(frame) => frame,
            Err(_) => {
                self.app_view.update_swap_chain();
                self.app_view
                    .swap_chain
                    .get_current_frame()
                    .expect("Failed to acquire next swap chain texture!")
            }
        };
        self.noise_display.draw(
            &frame.output.view,
            &mut encoder,
            wgpu::LoadOp::Clear(idroid::utils::alpha_color()),
        );
        self.app_view.queue.submit(Some(encoder.finish()));
    }
}
