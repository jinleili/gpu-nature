use idroid::{math::Position, math::TouchPoint, SurfaceView};
use nature::{CombinateCanvas, FieldAnimationType, FieldType, ParticleColorType, SettingObj};
#[cfg(not(target_arch = "wasm32"))]
use std::time::{Duration, Instant};
use uni_view::AppView;

fn main() {
    use winit::event::{
        ElementState, Event, KeyboardInput, MouseButton, MouseScrollDelta, VirtualKeyCode,
        WindowEvent,
    };
    use winit::{event_loop::ControlFlow, event_loop::EventLoop};
    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::init();
    };
    let events_loop = EventLoop::new();
    // let events_loop: EventLoop<(i32, String)> = EventLoop::with_user_event();
    // let proxy = events_loop.create_proxy();
    let size = winit::dpi::Size::Logical(winit::dpi::LogicalSize { width: 1000.0, height: 780.0 });

    let builder = winit::window::WindowBuilder::new().with_inner_size(size).with_title("Particles");
    let window = builder.build(&events_loop).unwrap();

    let v = pollster::block_on(AppView::new(window, false));
    // LBM Player
    let setting = SettingObj::new(
        FieldType::Fluid,
        FieldAnimationType::Poiseuille,
        ParticleColorType::MovementAngle,
        30000,
        0.0,
    );

    // field player
    // let setting = SettingObj::new(
    //     FieldType::Field,
    //     FieldAnimationType::JuliaSet,
    //     ParticleColorType::MovementAngle,
    //     30000,
    //     60.0,
    // );

    let mut surface_view = CombinateCanvas::new(v, setting);
    let mut last_update_inst = Instant::now();
    let mut last_touch_point: Position = Position::zero();
    events_loop.run(move |event, _, control_flow| {
        *control_flow = if cfg!(feature = "metal-auto-capture") {
            ControlFlow::Exit
        } else {
            ControlFlow::Poll
        };
        match event {
            Event::RedrawEventsCleared => {
                let target_frametime = Duration::from_secs_f64(1.0 / 60.0);
                let time_since_last_frame = last_update_inst.elapsed();
                if time_since_last_frame >= target_frametime {
                    surface_view.app_view.view.request_redraw();
                    last_update_inst = Instant::now();
                } else {
                    *control_flow = ControlFlow::WaitUntil(
                        Instant::now() + target_frametime - time_since_last_frame,
                    );
                }
            }
            Event::WindowEvent { event: WindowEvent::Resized(_size), .. } => {
                surface_view.resize();
            }
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            state: ElementState::Pressed,
                            ..
                        },
                    ..
                }
                | WindowEvent::CloseRequested => {
                    *control_flow = winit::event_loop::ControlFlow::Exit;
                }
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(VirtualKeyCode::W),
                            state: ElementState::Pressed,
                            ..
                        },
                    ..
                } => {
                    surface_view.reset();
                }
                WindowEvent::MouseWheel { delta, .. } => match delta {
                    MouseScrollDelta::LineDelta(_x, y) => {
                        println!("{:?}, {}", _x, y);
                    }
                    _ => (),
                },
                WindowEvent::MouseInput { device_id: _, state, button, .. } => {
                    match button {
                        MouseButton::Left => {
                            let point = TouchPoint::new_by_pos(Position::new(0.0, 0.0));

                            if state == ElementState::Pressed {
                                // surface_view.touch_start(point);
                                surface_view.on_click(last_touch_point);
                            } else {
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
                    // if left_bt_pressed {

                    // }
                    last_touch_point = Position::new(position.x as f32, position.y as f32);
                    let point = TouchPoint::new_by_pos(last_touch_point);
                    surface_view.touch_moved(point);
                }
                _ => {}
            },
            Event::RedrawRequested(_) => {
                surface_view.enter_frame();
            }
            _ => (),
        }
    });
}
