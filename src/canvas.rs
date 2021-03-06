use crate::Diffraction;

use app_surface::{
    math::{Position, Rect, Size},
    AppSurface, SurfaceFrame, Touch, TouchPhase,
};
pub struct Canvas {
    pub app_view: AppSurface,
    dc_origin: Position,
    depth_tex: crate::util::AnyTexture,
    d3_noise: crate::noise::D3NoiseTexture,
    nature_node: Diffraction,
    floor_node: crate::Floor,
    brick: crate::Brick,
}

#[allow(dead_code)]
impl Canvas {
    pub fn new(app_view: AppSurface) -> Self {
        let dc_origin =
            Position::new(app_view.config.width as f32 / 2.0, app_view.config.height as f32 / 2.0);
        let depth_tex = crate::util::load_texture::empty(
            &app_view.device,
            wgpu::TextureFormat::Depth32Float,
            wgpu::Extent3d {
                width: app_view.config.width,
                height: app_view.config.height,
                depth_or_array_layers: 1,
            },
            Some(wgpu::TextureViewDimension::D2),
            Some(wgpu::TextureUsages::RENDER_ATTACHMENT),
            Some("depth_tex"),
        );

        let d3_noise = crate::noise::D3NoiseTexture::create(&app_view);

        let nature_node = Diffraction::new(&app_view, true);

        // floor
        let floor_node = crate::Floor::new(&app_view, &d3_noise.tex, true);

        let brick = crate::Brick::new(&app_view);

        Self { app_view, dc_origin, depth_tex, d3_noise, nature_node, floor_node, brick }
    }
}

impl SurfaceFrame for Canvas {
    fn resize_surface(&mut self) {
        self.app_view.resize_surface();
    }
    fn touch(&mut self, touch: Touch) {
        match touch.phase {
            TouchPhase::Moved => {
                let mut p = touch.position.minus(&self.dc_origin);
                p.y *= -1.0;
                self.nature_node.rotate(
                    &self.app_view,
                    p.y / self.dc_origin.y,
                    -p.x / self.dc_origin.x,
                );
            }
            _ => {}
        }
    }

    fn enter_frame(&mut self) {
        // self.nature_node.enter_frame(&mut self.app_view);

        let (frame, frame_view) = self.app_view.get_current_frame_view();
        let color_attachments = [Some(wgpu::RenderPassColorAttachment {
            view: &frame_view,
            resolve_target: None,
            ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: true },
        })];
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
            // let mut rpass = encoder.begin_render_pass(&render_pass_descriptor);
            // self.floor_node.draw_render_pass(&mut rpass);
            // self.nature_node.draw_render_pass(&mut rpass);
        }
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &color_attachments,
                depth_stencil_attachment: None,
            });
            self.brick.display_node.draw_rpass(&mut rpass);
        }
        self.app_view.queue.submit(Some(encoder.finish()));
        frame.present();
    }
}
