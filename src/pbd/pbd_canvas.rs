use super::MaoBrush;

use app_surface::{math::Position, AppSurface, SurfaceFrame, Touch, TouchPhase};

pub struct PBDCanvas {
    dc_origin: Position,
    pub app_view: AppSurface,
    pub pbd_node: MaoBrush,
}

#[allow(dead_code)]
impl PBDCanvas {
    pub fn new(app_view: AppSurface) -> Self {
        let dc_origin =
            Position::new(app_view.config.width as f32 / 2.0, app_view.config.height as f32 / 2.0);
        let pbd_node = MaoBrush::new(&app_view);
        // let pbd_node = Line::new(&app_view);

        Self { dc_origin, app_view, pbd_node }
    }
}

impl SurfaceFrame for PBDCanvas {
    fn resize_surface(&mut self) {}
    fn touch(&mut self, touch: Touch) {
        match touch.phase {
            TouchPhase::Moved => {
                let mut p = touch.position.minus(&self.dc_origin);
                p.y *= -1.0;
                self.pbd_node.rotate(
                    &self.app_view,
                    p.y / self.dc_origin.y,
                    -p.x / self.dc_origin.x,
                );
            }
            _ => {}
        }
    }

    fn enter_frame(&mut self) {
        self.pbd_node.enter_frame(&mut self.app_view);
    }
}
