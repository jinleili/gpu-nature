use super::{Cloth, Cloth3, Line};

use idroid::{math::TouchPoint, SurfaceView};
use uni_view::AppView;

pub struct PBDCanvas {
    pub app_view: AppView,
    pub pbd_node: Cloth,
}

#[allow(dead_code)]
impl PBDCanvas {
    pub fn new(app_view: AppView) -> Self {
        let pbd_node = Cloth::new(&app_view);
        // let pbd_node = Line::new(&app_view);

        Self { app_view, pbd_node }
    }
}

impl SurfaceView for PBDCanvas {
    fn resize(&mut self) {}
    fn pintch_start(&mut self, _location: TouchPoint, _scale: f32) {}
    fn pintch_changed(&mut self, _location: TouchPoint, _scale: f32) {}
    fn touch_start(&mut self, _point: TouchPoint) {}
    fn touch_moved(&mut self, _point: TouchPoint) {}
    fn touch_end(&mut self, _point: TouchPoint) {}

    fn enter_frame(&mut self) {
        self.pbd_node.enter_frame(&mut self.app_view);
    }
}
