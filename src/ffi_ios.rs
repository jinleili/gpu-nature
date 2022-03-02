use crate::{CombinateCanvas, FieldAnimationType, FieldType, ParticleColorType, SettingObj};
use app_surface::{AppSurface, IOSViewObj, SurfaceFrame};

#[no_mangle]
pub unsafe extern "C" fn create_wgpu_canvas(ios_obj: IOSViewObj) -> *mut libc::c_void {
    let rust_view = AppSurface::new(ios_obj);
    let setting = SettingObj::new(
        FieldType::Fluid,
        FieldAnimationType::Poiseuille,
        ParticleColorType::Speed,
        30000,
        120.0,
    );
    let obj = CombinateCanvas::new(rust_view, setting);

    box_obj(obj)
}

#[no_mangle]
pub unsafe extern "C" fn enter_frame(obj: *mut libc::c_void) -> *mut libc::c_void {
    let mut obj: Box<Box<dyn SurfaceFrame>> = Box::from_raw(obj as *mut _);
    obj.enter_frame();

    Box::into_raw(obj) as *mut libc::c_void
}

fn box_obj(obj: impl SurfaceFrame) -> *mut libc::c_void {
    let boxed_trait: Box<dyn SurfaceFrame> = Box::new(obj);
    let boxed_boxed_trait = Box::new(boxed_trait);
    let heap_pointer = Box::into_raw(boxed_boxed_trait);
    // let boxed_boxed_trait = Box::new(v);
    // let heap_pointer = Box::into_raw(boxed_boxed_trait);
    heap_pointer as *mut libc::c_void
}
