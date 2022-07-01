use crate::util::node::{BufferlessFullscreenNode, ComputeNode, ViewNode, ViewNodeBuilder};
use zerocopy::{AsBytes, FromBytes};

#[repr(C)]
#[derive(Copy, Clone, AsBytes, FromBytes)]
struct BrickUniform {
    wh: [f32; 2],
    mortar_thickness: f32,
    _padding: f32,
    // brick + mortar thickness
    bm_wh: [f32; 2],
    // mortar half width and height within the brick
    mortar_half_wh: [f32; 2],
}

pub struct Brick {
    pub display_node: BufferlessFullscreenNode,
}

impl Brick {
    pub fn new(app_view: &app_surface::AppSurface) -> Self {
        let shader =
            crate::util::shader::create_shader_module(&app_view.device, "procedural/brick", None);
        let width = 108.0 * app_view.scale_factor;
        let height = width / 2.3;
        let mortar = width / 18.0;
        let bm_wh = [width + mortar, height + mortar];
        let uniform_data = BrickUniform {
            wh: [width, height],
            mortar_thickness: mortar,
            bm_wh,
            mortar_half_wh: [(mortar * 0.5) / bm_wh[0], (mortar * 0.5) / bm_wh[1]],
            _padding: 0.0,
        };

        let buf =
            crate::util::BufferObj::create_uniform_buffer(&app_view.device, &uniform_data, None);
        let display_node = BufferlessFullscreenNode::new(
            &app_view.device,
            app_view.config.format,
            vec![&buf],
            vec![],
            vec![],
            vec![],
            &shader,
            None,
            false,
        );

        Self { display_node }
    }
}
