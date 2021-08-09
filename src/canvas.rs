use idroid::{math::TouchPoint, SurfaceView};
use uni_view::AppView;
use crate::Diffraction;
pub struct Canvas {
    pub app_view: AppView,
    pub nature_node: Diffraction,
}

#[allow(dead_code)]
impl Canvas {
    pub fn new(app_view: AppView) -> Self {
        
        let nature_node = Diffraction::new(&app_view);
        Self { app_view, nature_node }
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
        self.nature_node.enter_frame(&mut self.app_view);
    }
}
