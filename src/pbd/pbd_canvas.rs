use super::{MaoBrush};

use idroid::{math::TouchPoint, SurfaceView};
use uni_view::AppView;

pub struct PBDCanvas {
    dc_origin: idroid::math::Position,
    pub app_view: AppView,
    pub pbd_node: MaoBrush,
}

#[allow(dead_code)]
impl PBDCanvas {
    pub fn new(app_view: AppView) -> Self {
        let dc_origin = idroid::math::Position::new(
            app_view.config.width as f32 / 2.0,
            app_view.config.height as f32 / 2.0,
        );
        let pbd_node = MaoBrush::new(&app_view);
        // let pbd_node = Line::new(&app_view);

        Self { dc_origin, app_view, pbd_node }
    }
}

impl SurfaceView for PBDCanvas {
    fn resize(&mut self) {}
    fn pintch_start(&mut self, _location: TouchPoint, _scale: f32) {}
    fn pintch_changed(&mut self, _location: TouchPoint, _scale: f32) {}
    fn touch_start(&mut self, _point: TouchPoint) {}

    fn touch_moved(&mut self, point: TouchPoint) {
        let mut p = point.pos.minus(&self.dc_origin);
        p.y *= -1.0;
        self.pbd_node.rotate(&self.app_view, p.y / self.dc_origin.y, -p.x / self.dc_origin.x);
    }

    fn touch_end(&mut self, _point: TouchPoint) {}

    fn enter_frame(&mut self) {
        self.pbd_node.enter_frame(&mut self.app_view);
    }
}
