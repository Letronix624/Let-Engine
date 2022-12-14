//#![windows_subsystem = "windows"]

mod consts;
mod data;
mod game;
mod server;
mod vulkan;

extern crate image;
extern crate vulkano;
use data::*;
use game::{Game, Object};
use server::Server;
use std::collections::HashMap;
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread::{self, sleep};
use std::time::*;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, VirtualKeyCode};
use winit::{
    event::{Event, WindowEvent},
    event_loop::ControlFlow,
    window::Window,
};

lazy_static::lazy_static! {
    static ref GAME: Mutex<Game> = Mutex::new(Game::init());
}

static mut FPS: u16 = 0;
static mut DELTA_TIME: f64 = 0.0;

#[allow(dead_code)]
fn delta_time() -> f64 {
    unsafe {
        return DELTA_TIME;
    }
}
#[allow(dead_code)]
fn fps() -> u16 {
    unsafe {
        return FPS;
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let server_mode = args.contains(&"--server".to_string());
    if server_mode {
        match server() {
            Ok(_) => (),
            Err(e) => {
                println!("Server closed! Reason: {e}")
            }
        };
    } else {
        client();
    }
}

fn server() -> std::io::Result<()> {
    //start
    let socket = Arc::new(Mutex::new(match Server::init() {
        Ok(t) => t,
        Err(e) => {
            panic!("Couldn't start the server: {}", e);
        }
    }));

    socket.clone().lock().unwrap().start()?;

    //tick (62.4/s)
    let serv = socket.clone();
    thread::spawn(move || loop {
        serv.lock().unwrap().broadcastobjs().unwrap();
        sleep(Duration::from_nanos(16025641));
    });
    //main
    let listener: TcpListener;
    {
        let soc = socket.clone();
        let sock = soc.lock().unwrap();
        listener = TcpListener::bind(format!("{}:{}", &sock.ip, &sock.port))?;
    }

    for stream in listener.incoming() {
        let conn = stream?.try_clone()?;
        let addr = conn.try_clone()?;
        let addr = addr.peer_addr()?;
        thread::spawn(move || Server::tcpconnection(conn, addr));
    }
    Ok(())
}

fn client() {
    // let game = App::initialize();
    // GAME.lock().unwrap()mainloop();
    GAME.lock().unwrap().start();
    thread::spawn(|| {
        loop {
            {
                let mut game = GAME.lock().unwrap();
                game.tick();
            }
            //thread::sleep(Duration::from_secs(1));
            thread::sleep(Duration::from_nanos(16025641));
        }
    });
    let (mut app, event_loop) = vulkan::App::initialize();
    let mut dt = unix_timestamp();
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            //Event::
            Event::WindowEvent {
                event: WindowEvent::Resized(_),
                ..
            } => {
                app.recreate_swapchain = true;
            }
            Event::WindowEvent {
                event: WindowEvent::MouseInput { button, state, .. },
                ..
            } => {
                match button {
                    MouseButton::Left => {
                        GAME.lock().unwrap().input.lmb = state == ElementState::Pressed;
                    }
                    MouseButton::Middle => {
                        GAME.lock().unwrap().input.mmb = state == ElementState::Pressed;
                    }
                    MouseButton::Right => {
                        GAME.lock().unwrap().input.rmb = state == ElementState::Pressed;
                    }
                    MouseButton::Other(_t) => {
                        //println!("{_t}");
                    }
                }
            }
            Event::WindowEvent {
                event: WindowEvent::MouseWheel { delta, .. },
                ..
            } => match delta {
                MouseScrollDelta::LineDelta(.., t) => {
                    GAME.lock().unwrap().input.vsd = t;
                }
                _ => (),
            },
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput { input, .. },
                ..
            } => {
                if let Some(key_code) = input.virtual_keycode {
                    match key_code {
                        VirtualKeyCode::F11 => {
                            if input.state == ElementState::Released {
                                vulkan::App::fullscreen(app.surface.clone());
                            }
                        }
                        VirtualKeyCode::W => {
                            GAME.lock().unwrap().input.w = input.state == ElementState::Pressed;
                        }
                        VirtualKeyCode::A => {
                            GAME.lock().unwrap().input.a = input.state == ElementState::Pressed;
                        }
                        VirtualKeyCode::S => {
                            GAME.lock().unwrap().input.s = input.state == ElementState::Pressed;
                        }
                        VirtualKeyCode::D => {
                            GAME.lock().unwrap().input.d = input.state == ElementState::Pressed;
                        }
                        VirtualKeyCode::Q => {
                            GAME.lock().unwrap().input.q = input.state == ElementState::Pressed;
                        }
                        VirtualKeyCode::E => {
                            GAME.lock().unwrap().input.e = input.state == ElementState::Pressed;
                        }
                        VirtualKeyCode::R => {
                            GAME.lock().unwrap().input.r = input.state == ElementState::Pressed;
                        }
                        VirtualKeyCode::Escape => {
                            if input.state == ElementState::Pressed {
                                *control_flow = ControlFlow::Exit;
                            }
                        }

                        _ => (),
                    }
                }
            }
            Event::WindowEvent {
                event: WindowEvent::CursorMoved { position, .. },
                ..
            } => {
                let dim = app
                    .surface
                    .object()
                    .unwrap()
                    .downcast_ref::<Window>()
                    .unwrap()
                    .inner_size();
                GAME.lock().unwrap().input.mouse = (
                    (position.x as f32 / dim.width as f32) * 2.0 - 1.0,
                    (position.y as f32 / dim.height as f32) * 2.0 - 1.0,
                )
            }
            Event::MainEventsCleared => {
                //game stuff early update
                unsafe {
                    DELTA_TIME = unix_timestamp() - dt;
                    dt = unix_timestamp();
                    FPS = (1.0 / DELTA_TIME) as u16;
                }

                let objects: HashMap<String, Object>;
                objects = GAME.lock().unwrap().objects.clone();

                app.vertices = vec![];
                app.player = objects.get("player1").unwrap().clone();

                for i in GAME
                    .lock()
                    .unwrap()
                    .renderorder
                    .iter()
                    .map(|x| objects.get(x).unwrap())
                {
                    app.vertices.append(&mut i.data.clone());
                }

                // for obj in GAME
                //     .lock()
                //     .unwrap()
                //     .renderorder
                //     .iter()
                //     .map(|x| objects.get(x).unwrap())
                // {
                //     for vertex in obj.data.iter() {
                //         let hypo = vertex.position[0].hypot(vertex.position[1]);
                //         let rotatedpos: [f32; 2] = [
                //             (f32::atan2(vertex.position[1], vertex.position[0]) + obj.rotation)
                //                 .cos()
                //                 * hypo, // ???(2) ?? 2 ?? ???(2)
                //             (f32::atan2(vertex.position[1], vertex.position[0]) + obj.rotation)
                //                 .sin()
                //                 * hypo, //  hypo  /// x = cos(cos-1(vx : sqrt(vx^2 + vy^2) + obj.rotation)) * hypo, ;
                //         ];
                //         app.vertices.push(Vertex {
                //             position: [
                //                 rotatedpos[0] * obj.size[0] + obj.position[0],
                //                 rotatedpos[1] * obj.size[1] + obj.position[1],
                //             ],
                //         });
                //     }
                // }

                GAME.lock().unwrap().main();
            }
            Event::RedrawEventsCleared => {
                app.redrawevent();
                GAME.lock().unwrap().late_main();
            }
            _ => (),
        }
    });
}
fn unix_timestamp() -> f64 {
    return SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs_f64();
}
