use nature::PBDCanvas;
use idroid::math::{Position, TouchPoint};
use idroid::SurfaceView;
use uni_view::AppView;

use std::cell::RefCell;
use std::rc::Rc;
// use serde::{Deserialize, Serialize};

fn main() {
    use winit::event::{ElementState, Event, KeyboardInput, MouseScrollDelta, VirtualKeyCode, WindowEvent};
    use winit::{
        event_loop::{ControlFlow, EventLoop},
        window::Window,
    };

    env_logger::init();
    let events_loop = EventLoop::new();
    let (window, size) = {
        let window = Window::new(&events_loop).unwrap();
        let size = winit::dpi::Size::Logical(winit::dpi::LogicalSize { width: 400.0, height: 800.0 });
        // 设置 set_inner_size 后，窗口尺寸会在前几帧有变化
        window.set_inner_size(size);
        window.set_max_inner_size(Some(size));
        window.set_title("pbd");
        (window, size)
    };

    let mut v = pollster::block_on(AppView::new(window, true));
    if cfg!(target_os = "macos") {
        let temporary_directory: &'static str =
            Box::leak(format!("{}/assets/", &env!("CARGO_MANIFEST_DIR")).into_boxed_str());
        v.library_directory = temporary_directory;
    };
    let mut surface_view: PBDCanvas = PBDCanvas::new(v);

    let mut current_index: usize = 0;
    // 窗口在刚出来的几帧，view port 的尺寸是在变化调整中的，不是最终值
    let mut init_index: usize = 6;
    let mut frame_index = 0;
    events_loop.run(move |event, _, control_flow| {
        *control_flow = if cfg!(feature = "metal-auto-capture") { ControlFlow::Exit } else { ControlFlow::Poll };
        match event {
            Event::MainEventsCleared => surface_view.app_view.view.request_redraw(),
            Event::WindowEvent { event: WindowEvent::Resized(_size), .. } => {
                surface_view.resize();
            }
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(VirtualKeyCode::Escape), state: ElementState::Pressed, ..
                        },
                    ..
                }
                | WindowEvent::CloseRequested => {
                    *control_flow = winit::event_loop::ControlFlow::Exit;
                }
                WindowEvent::MouseWheel { delta, .. } => match delta {
                    MouseScrollDelta::LineDelta(_x, y) => {
                        println!("{:?}, {}", _x, y);
                    }
                    _ => (),
                },
                WindowEvent::Touch(touch) => {
                    println!("{:?}", touch);
                }
                WindowEvent::CursorMoved { position, .. } => {
                    // let point = TouchPoint { pos: Position::new(position.x as f32, position.y as f32), force: 0.0 };
                    // surface_view.touch_moved(point);
                }
                _ => {}
            },
            Event::RedrawRequested(_) => {
                // 减慢渲染速度
                frame_index += 1;
                if frame_index % 10000 > 0 {
                    return ();
                }
                current_index += 1;
                if current_index <= 1000 {
                    surface_view.enter_frame();
                }
            }
            _ => (),
        }
    });
}
