use crate::{setting_obj::SettingObj, D3FluidPlayer, FieldPlayer, FieldType, FluidPlayer, Player};
use idroid::{
    math::{Size, TouchPoint},
    BufferObj, SurfaceView,
};
use uni_view::{AppView, GPUContext};
use zerocopy::AsBytes;

pub struct CombinateCanvas {
    pub app_view: AppView,
    canvas_size: Size<u32>,
    canvas_buf: BufferObj,
    setting: SettingObj,
    player: Box<dyn Player>,
}

impl CombinateCanvas {
    pub fn new(app_view: AppView, setting: SettingObj) -> Self {
        let test_shader = idroid::shader::create_shader_module(
            &app_view.device,
            "aa_lbm/aa_collide_stream",
            Some("aa_collide_stream"),
        );
        let canvas_size: Size<u32> = (&app_view.config).into();
        let mut setting = setting;
        setting.update_canvas_size(&app_view.device, &app_view.queue, canvas_size);
        let canvas_buf = idroid::BufferObj::create_empty_storage_buffer(
            &app_view.device,
            (canvas_size.width * canvas_size.height * 12) as u64,
            false,
            Some("canvas_buf"),
        );

        let player = Self::create_player(&app_view, canvas_size, &canvas_buf, &setting);
        CombinateCanvas { app_view, canvas_size, canvas_buf, setting, player }
    }

    pub fn update_field_type(
        &mut self, field_ty: FieldType, animation_ty: crate::FieldAnimationType,
    ) {
        if self.setting.update_field_type(&self.app_view.queue, field_ty) {
            self.setting.animation_type = animation_ty;
            self.recreate_player();
        }
    }

    pub fn update_fluid_viscosity(&mut self, nu: f32) {
        if self.setting.field_type == FieldType::Fluid && self.setting.fluid_viscosity != nu {
            self.setting.fluid_viscosity = nu;
            self.player.update_uniforms(&self.app_view.queue, &self.setting);
        }
    }

    pub fn update_particles_count(&mut self, count: i32) {
        self.setting.update_particles_count(&self.app_view.device, &self.app_view.queue, count);
    }

    pub fn update_particle_color(&mut self, color_type: crate::ParticleColorType) {
        self.setting.update_particle_color(&self.app_view.device, &self.app_view.queue, color_type);
    }

    pub fn update_particle_point_size(&mut self, point_size: i32) {
        self.setting.update_particle_point_size(&self.app_view.queue, point_size);
    }

    pub fn update_animation_type(&mut self, ty: crate::FieldAnimationType) {
        self.setting.animation_type = ty;
        self.recreate_player();
    }

    pub fn recreate_player(&mut self) {
        self.player =
            Self::create_player(&self.app_view, self.canvas_size, &self.canvas_buf, &self.setting);
    }

    pub fn reset(&mut self) {
        self.player.reset(&self.app_view.device, &self.app_view.queue);
    }

    pub fn on_click(&mut self, pos: idroid::math::Position) {
        self.player.on_click(&self.app_view.device, &self.app_view.queue, pos);
    }

    fn create_player<'a>(
        app_view: &AppView, canvas_size: Size<u32>, canvas_buf: &idroid::BufferObj,
        setting: &SettingObj,
    ) -> Box<dyn Player> {
        return match setting.field_type {
            FieldType::Field => Box::new(FieldPlayer::new(
                &app_view.device,
                &app_view.queue,
                app_view.config.format,
                canvas_size,
                canvas_buf,
                setting,
            )),
            FieldType::Fluid => Box::new(FluidPlayer::new(
                &app_view.device,
                &app_view.queue,
                app_view.config.format,
                canvas_size,
                canvas_buf,
                setting,
            )),
            _ => Box::new(D3FluidPlayer::new(
                &app_view.device,
                &app_view.queue,
                app_view.config.format,
                canvas_size,
                canvas_buf,
                setting,
            )),
        };
    }
}

impl SurfaceView for CombinateCanvas {
    fn touch_start(&mut self, _point: TouchPoint) {
        self.player.touch_begin(&self.app_view.device, &self.app_view.queue);
    }
    fn touch_moved(&mut self, p: TouchPoint) {
        self.player.touch_move(&self.app_view.device, &self.app_view.queue, p.pos);
    }
    fn touch_end(&mut self, _point: TouchPoint) {
        self.player.touch_end(&self.app_view.device, &self.app_view.queue);
    }

    fn resize(&mut self) {
        self.app_view.resize_surface();
        self.canvas_size = (&self.app_view.config).into();
        self.canvas_buf = idroid::BufferObj::create_empty_storage_buffer(
            &self.app_view.device,
            (self.canvas_size.width * self.canvas_size.height * 12) as u64,
            false,
            Some("canvas_buf"),
        );
        self.recreate_player();
    }

    fn enter_frame(&mut self) {
        let (_frame, frame_view) = self.app_view.get_current_frame_view();
        self.player.enter_frame(
            &self.app_view.device,
            &self.app_view.queue,
            &frame_view,
            &mut self.setting,
        );
    }
}
