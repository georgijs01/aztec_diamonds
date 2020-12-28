#![feature(or_patterns)]

use std::time::{Duration, Instant};
use winit::event::{Event, KeyboardInput, StartCause, VirtualKeyCode, WindowEvent};
use winit::event_loop::ControlFlow;

mod grid;

const WINDOW_SIZE: u32 = 1024;

fn main() -> Result<(), pixels::Error> {
    struct UpdateEvent;
    let event_loop = winit::event_loop::EventLoop::<UpdateEvent>::with_user_event();
    let proxy = event_loop.create_proxy();
    let window = {
        let size = winit::dpi::LogicalSize::new(WINDOW_SIZE as f64, WINDOW_SIZE as f64);
        winit::window::WindowBuilder::new()
            .with_title("Aztec Diamonds")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .with_resizable(false)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture =
            pixels::SurfaceTexture::new(window_size.width, window_size.height, &window);
        pixels::Pixels::new(WINDOW_SIZE, WINDOW_SIZE, surface_texture)?
    };

    let mut show = grid::ShowGrid::new(WINDOW_SIZE);

    let mut update_interval = Duration::from_millis(300);
    let mut paused = false;
    let mut half_step_mode = false;

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::NewEvents(StartCause::Init) => {
                *control_flow = ControlFlow::WaitUntil(Instant::now() + update_interval);
            }
            Event::NewEvents(StartCause::ResumeTimeReached { .. }) => {
                proxy.send_event(UpdateEvent).ok();
                *control_flow = ControlFlow::WaitUntil(Instant::now() + update_interval);
            }
            Event::UserEvent(UpdateEvent) => {
                if !paused {
                    if half_step_mode {
                        show.half_step();
                    } else {
                        show.full_step();
                    }
                    window.request_redraw();
                }
            }
            Event::WindowEvent {
                event:
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    },
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode: Some(VirtualKeyCode::Space),
                                state: winit::event::ElementState::Pressed,
                                ..
                            },
                        ..
                    },
                ..
            } => {
                if !paused {
                    paused = true;
                } else {
                    show.half_step();
                    window.request_redraw();
                }
            }
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode: Some(VirtualKeyCode::P),
                                state: winit::event::ElementState::Pressed,
                                ..
                            },
                        ..
                    },
                ..
            } => {
                paused = !paused;
            }
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode: Some(VirtualKeyCode::Up),
                                state: winit::event::ElementState::Pressed,
                                ..
                            },
                        ..
                    },
                ..
            } => {
                update_interval = update_interval / 12 * 10;
            }
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode: Some(VirtualKeyCode::Down),
                                state: winit::event::ElementState::Pressed,
                                ..
                            },
                        ..
                    },
                ..
            } => {
                update_interval = update_interval / 10 * 12;
            }
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode: Some(VirtualKeyCode::R),
                                state: winit::event::ElementState::Pressed,
                                ..
                            },
                        ..
                    },
                ..
            } => {
                update_interval = Duration::from_millis(300);
            }
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode: Some(VirtualKeyCode::M),
                                state: winit::event::ElementState::Pressed,
                                ..
                            },
                        ..
                    },
                ..
            } => {
                half_step_mode = !half_step_mode;   
            }
            Event::RedrawRequested(_) => {
                show.draw(pixels.get_frame());
                if let Err(_) = pixels.render() {
                    *control_flow = winit::event_loop::ControlFlow::Exit;
                }
            }
            _ => ()
        }
    });
}
