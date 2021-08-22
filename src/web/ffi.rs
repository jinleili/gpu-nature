use crate::{CombinateCanvas, FieldAnimationType, FieldType, ParticleColorType, SettingObj};
use idroid::{math::Position, math::TouchPoint, SurfaceView};
use uni_view::{AppView, GPUContext};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::console;
use winit::platform::web::WindowExtWebSys;
use winit::{
    event::{ElementState, Event, MouseButton, MouseScrollDelta, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

const FIELD_TYPE: &'static str = "field_type";
const FIELD_ANIMATION_TYPE: &'static str = "field_animation_type";
const FLUID_VISCOSITY_CHANGED: &'static str = "fluid_viscosity_changed";

const PARTICLE_COLOR_CHANGED: &'static str = "particle_color_changed";
const PARTICLES_COUNT_CHANGED: &'static str = "particles_count_changed";
const PARTICLE_SIZE_CHANGED: &'static str = "particle_size_changed";

const CANVAS_SIZE_NEED_CHANGE: &'static str = "canvas_size_need_change";
const CANVAS_ANIMATION_RESUME: &'static str = "canvas_animation_resume";
const CANVAS_ANIMATION_SUSPEND: &'static str = "canvas_animation_suspend";

const CANVAS_RESET: &'static str = "canvas_reset";

const ALL_CUSTOM_EVENTS: [&'static str; 10] = [
    FIELD_TYPE,
    FIELD_ANIMATION_TYPE,
    FLUID_VISCOSITY_CHANGED,
    PARTICLE_COLOR_CHANGED,
    PARTICLES_COUNT_CHANGED,
    PARTICLE_SIZE_CHANGED,
    CANVAS_SIZE_NEED_CHANGE,
    CANVAS_ANIMATION_RESUME,
    CANVAS_ANIMATION_SUSPEND,
    CANVAS_RESET,
];

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(a: &str);
    fn change_canvas_size();
    fn canvas_resize_completed();
}

#[macro_export]
macro_rules! console_log {
    ($($t:tt)*) => (crate::web::log(&format_args!($($t)*).to_string()))
}

#[derive(Debug)]
struct ParticleSettingEvent {
    ty: &'static str,
    value: String,
}

#[wasm_bindgen(start)]
pub fn run() {
    console_log::init_with_level(log::Level::Debug);

    let event_loop: EventLoop<ParticleSettingEvent> = EventLoop::with_user_event();
    let proxy = event_loop.create_proxy();

    let size = winit::dpi::Size::Logical(winit::dpi::LogicalSize { width: 1000.0, height: 800.0 });
    let window = WindowBuilder::new()
        .with_inner_size(size)
        .with_title("gpu-particles-rs")
        .build(&event_loop)
        .unwrap();
    let canvas = window.canvas();

    let web_window = web_sys::window().unwrap();
    let storage: web_sys::Storage = web_window.local_storage().unwrap().unwrap();
    let host = web_window.location().host().unwrap();

    let document = web_window.document().unwrap();
    let container = document.get_element_by_id("canvas_container").unwrap();
    container.append_child(&canvas).expect("Append canvas to HTML body");

    let target: web_sys::EventTarget = container.into();
    let call_back = Closure::wrap(Box::new(move |event: web_sys::Event| {
        // let event_name: &'static str = event.type_().as_str();
        let event_name: &'static str = Box::leak(event.type_().into_boxed_str());
        proxy.send_event(ParticleSettingEvent { ty: event_name, value: String::new() });
    }) as Box<dyn FnMut(_)>);

    // Add html element's event listener
    for e in ALL_CUSTOM_EVENTS.iter() {
        target.add_event_listener_with_callback(e, call_back.as_ref().unchecked_ref()).unwrap();
    }
    call_back.forget();

    change_canvas_size();

    let particles_count = storage.get_item("particles_count").unwrap().unwrap();
    let particle_lifetime = 60.0;
    let setting = SettingObj::new(
        FieldType::Fluid,
        FieldAnimationType::Poiseuille,
        ParticleColorType::MovementAngle,
        particles_count.parse::<i32>().unwrap(),
        particle_lifetime,
    );

    wasm_bindgen_futures::spawn_local(async move {
        let v = AppView::new(window).await;
        let mut surface_view = CombinateCanvas::new(v, setting);

        let container = document.get_element_by_id("canvas_container").unwrap();
        let mut last_size: f64 = 0.0;
        let mut left_bt_pressed = false;
        let mut last_touch_point: Position = Position::zero();

        event_loop.run(move |event, _, control_flow| {
            // if set *control_flow = ControlFlow::Wait, window cann't excute requestAnimationFrame automaticly
            *control_flow = ControlFlow::Poll;
            match event {
                Event::WindowEvent { event: WindowEvent::CloseRequested, window_id } => {
                    if surface_view.app_view.view.id() == window_id {
                        *control_flow = ControlFlow::Exit
                    }
                }
                Event::RedrawEventsCleared => {
                    surface_view.app_view.view.request_redraw();
                }
                Event::WindowEvent { event: WindowEvent::Resized(_size), .. } => {
                    surface_view.resize();
                    // Notify js that the resize has been completed
                    canvas_resize_completed();

                    console_log!("user event: -- resize -- ");
                }
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::MouseWheel { delta, .. } => match delta {
                        MouseScrollDelta::LineDelta(_x, y) => {
                            println!("{:?}, {}", _x, y);
                        }
                        _ => (),
                    },
                    WindowEvent::MouseInput { device_id, state, button, .. } => {
                        match button {
                            MouseButton::Left => {
                                let point = TouchPoint::new_by_pos(Position::new(0.0, 0.0));

                                if state == ElementState::Pressed {
                                    // left_bt_pressed = true;
                                    // surface_view.touch_start(point);
                                    surface_view.on_click(last_touch_point);
                                } else {
                                    left_bt_pressed = false;
                                    surface_view.touch_end(point);
                                }
                            }
                            _ => (),
                        };
                    }
                    WindowEvent::Touch(touch) => {
                        println!("{:?}", touch);
                    }
                    WindowEvent::CursorMoved { position, .. } => {
                        last_touch_point = Position::new(position.x as f32, position.y as f32);
                        let point = TouchPoint::new_by_pos(last_touch_point);
                        surface_view.touch_moved(point);
                    }
                    _ => {}
                },
                Event::UserEvent(event) => {
                    match event.ty {
                        FIELD_TYPE => {
                            let val = storage.get_item("field_type").unwrap().unwrap();
                            let ani_ty = storage.get_item("field_animation_type").unwrap().unwrap();
                            let animation_ty = get_animation_type(ani_ty);
                            let field_ty = match val.as_str() {
                                "1" => crate::FieldType::Field,
                                "2" => crate::FieldType::Fluid,
                                _ => crate::FieldType::Field,
                            };
                            surface_view.update_field_type(field_ty, animation_ty);
                        }
                        FIELD_ANIMATION_TYPE => {
                            let val = storage.get_item("field_animation_type").unwrap().unwrap();
                            surface_view.update_animation_type(get_animation_type(val));
                        }
                        FLUID_VISCOSITY_CHANGED => {
                            let val = storage.get_item("fluid_viscosity").unwrap().unwrap();
                            if let Ok(nu) = val.parse::<f32>() {
                                surface_view.update_fluid_viscosity(nu);
                            }
                        }
                        PARTICLES_COUNT_CHANGED => {
                            let val = storage.get_item("particles_count").unwrap().unwrap();
                            if let Ok(count) = val.parse::<i32>() {
                                surface_view.update_particles_count(count);
                            }
                        }
                        PARTICLE_SIZE_CHANGED => {
                            let val = storage.get_item("particle_size").unwrap().unwrap();
                            if let Ok(point_size) = val.parse::<i32>() {
                                surface_view.update_particle_point_size(point_size);
                            }
                        }
                        PARTICLE_COLOR_CHANGED => {
                            let val = storage.get_item("color_type").unwrap().unwrap();
                            let color_type = match val.as_str() {
                                "1" => crate::ParticleColorType::MovementAngle,
                                "2" => crate::ParticleColorType::Speed,
                                _ => crate::ParticleColorType::Uniform,
                            };
                            surface_view.update_particle_color(color_type);
                        }
                        CANVAS_ANIMATION_RESUME => {
                            // *control_flow = ControlFlow::Poll;
                        }
                        CANVAS_SIZE_NEED_CHANGE => {
                            // need_poll = false;
                            // *control_flow = ControlFlow::Wait;

                            // change canvas size on JS/HTML side will cause device lost
                            // let rect = container.get_bounding_client_rect();
                            // let w = rect.width();
                            // let h = rect.height();
                            // if w > 300.0 && h > 300.0 && ((w + h) - last_size).abs() > 20.0 {
                            //     last_size = w + h;
                            //     surface_view.app_view.set_view_size((w, h));
                            //     console_log!("user event: set_view_size: {}, {}", w, h);
                            // }
                        }
                        CANVAS_RESET => {
                            surface_view.reset();
                        }
                        _ => (),
                    }

                    console_log!("user event: {:?}", event)
                }
                Event::RedrawRequested(_) => {
                    // console_log!("{:?}", &host);
                    // if host.contains("localhost") || host.contains("jinleili.github") {
                    //     surface_view.enter_frame();
                    // }
                    surface_view.enter_frame();
                }
                _ => (),
            }
        });
    });
}

fn get_animation_type(val: String) -> FieldAnimationType {
    match val.as_str() {
        "1" => FieldAnimationType::Spirl,
        "2" => FieldAnimationType::JuliaSet,
        "3" => FieldAnimationType::Poiseuille,
        "4" => FieldAnimationType::Custom,
        _ => FieldAnimationType::Basic,
    }
}
