use app_surface::{math::Position, AppSurface, SurfaceFrame, Touch};
use nature::Canvas;
#[cfg(not(target_arch = "wasm32"))]
use std::time::{Duration, Instant};

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
    let size = winit::dpi::Size::Logical(winit::dpi::LogicalSize { width: 800.0, height: 800.0 });

    let builder =
        winit::window::WindowBuilder::new().with_inner_size(size).with_title("gpu-nature");
    let window = builder.build(&events_loop).unwrap();

    let v = AppSurface::new(window);

    // let mut surface_view = CombinateCanvas::new(v);
    let mut surface_view = Canvas::new(v);

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
                surface_view.resize_surface();
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
                } => {}
                WindowEvent::MouseWheel { delta, .. } => match delta {
                    MouseScrollDelta::LineDelta(_x, y) => {
                        println!("{:?}, {}", _x, y);
                    }
                    _ => (),
                },
                WindowEvent::MouseInput { device_id: _, state, button, .. } => {
                    match button {
                        MouseButton::Left => {
                            let point = Touch::touch_end(Position::new(0.0, 0.0));

                            if state == ElementState::Pressed {
                                // surface_view.touch_start(point);
                            } else {
                                surface_view.touch(point);
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
                    let point = Touch::touch_move(last_touch_point);
                    surface_view.touch(point);
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
