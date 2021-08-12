use crate::noise::*;
use crate::Diffraction;
use idroid::node::{BufferlessFullscreenNode, ComputeNode, ImageNodeBuilder, ImageViewNode};
use idroid::{math::Position, math::TouchPoint, SurfaceView};
use uni_view::{AppView, GPUContext};
pub struct Canvas {
    pub app_view: AppView,
    dc_origin: Position,
    depth_tex: idroid::AnyTexture,
    nature_node: Diffraction,
    floor_node: crate::Floor,
}

#[allow(dead_code)]
impl Canvas {
    pub fn new(app_view: AppView) -> Self {
        let dc_origin =
            Position::new(app_view.config.width as f32 / 2.0, app_view.config.height as f32 / 2.0);
        let depth_tex = idroid::load_texture::empty(
            &app_view.device,
            wgpu::TextureFormat::Depth32Float,
            wgpu::Extent3d {
                width: app_view.config.width,
                height: app_view.config.height,
                depth_or_array_layers: 1,
            },
            Some(wgpu::TextureViewDimension::D2),
            Some(wgpu::TextureUsages::RENDER_ATTACHMENT),
        );
        let nature_node = Diffraction::new(&app_view, true);

        // floor
        let floor_node = crate::Floor::new(&app_view, true);

        Self { app_view, dc_origin, depth_tex, nature_node, floor_node }
    }
}

impl SurfaceView for Canvas {
    fn resize(&mut self) {
        self.app_view.resize_surface();
    }
    fn pintch_start(&mut self, _location: TouchPoint, _scale: f32) {}
    fn pintch_changed(&mut self, _location: TouchPoint, _scale: f32) {}
    fn touch_start(&mut self, _point: TouchPoint) {}
    fn touch_end(&mut self, _point: TouchPoint) {}

    fn touch_moved(&mut self, point: TouchPoint) {
        let mut p = point.pos.minus(&self.dc_origin);
        p.y *= -1.0;
        self.nature_node.rotate(&self.app_view, p.y / self.dc_origin.y, -p.x / self.dc_origin.x);
    }

    fn enter_frame(&mut self) {
        // self.nature_node.enter_frame(&mut self.app_view);

        let (_frame, frame_view) = self.app_view.get_current_frame_view();
        let color_attachments = [wgpu::RenderPassColorAttachment {
            view: &frame_view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.2, g: 0.2, b: 0.3, a: 1.0 }),
                store: true,
            },
        }];
        let render_pass_descriptor = wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &color_attachments,
            // depth_stencil_attachment: None,
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_tex.tex_view,
                depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(1.0), store: false }),
                stencil_ops: None,
            }),
        };

        let mut encoder =
            self.app_view.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("diffraction encoder"),
            });
        {
            let mut rpass = encoder.begin_render_pass(&render_pass_descriptor);
            self.floor_node.draw_render_pass(&mut rpass);
            self.nature_node.draw_render_pass(&mut rpass);
        }
        self.app_view.queue.submit(Some(encoder.finish()));
    }
}
