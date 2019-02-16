#[macro_use]
extern crate glium;
extern crate evo_view_model;

use glium::{glutin, Surface};

pub mod background_drawing;
pub mod bullseye_drawing;

use background_drawing::*;
use bullseye_drawing::*;

fn _main() {
    let mut events_loop = glutin::EventsLoop::new();
    let window = glutin::WindowBuilder::new()
        .with_dimensions(glutin::dpi::LogicalSize::new(500.0, 500.0));
    let context = glutin::ContextBuilder::new();
    let display = glium::Display::new(window, context, &events_loop).unwrap();

    let world_dim = [200.0, 100.0];

    let background_drawing = BackgroundDrawing::new(&display);
    let world = vec![
        World { dimensions: world_dim, top_color: [0.0, 0.1, 0.5], bottom_color: [0.0, 0.0, 0.0] }
    ];
    let world_vb = glium::VertexBuffer::new(&display, &world).unwrap();

    let bullseye_drawing = BullseyeDrawing::new(&display);
    let bullseyes = vec![
        Bullseye { center: [-50.0, -30.0], radius: 10.0 },
        Bullseye { center: [-50.0, 30.0], radius: 5.0 },
        Bullseye { center: [50.0, -30.0], radius: 15.0 },
        Bullseye { center: [50.0, 30.0], radius: 20.0 },
    ];
    let bullseyes_vb = glium::VertexBuffer::new(&display, &bullseyes).unwrap();

    let mut closed = false;
    while !closed {
        let window_size = display.gl_window().get_inner_size().unwrap();
        let window_dim = [window_size.width as f32, window_size.height as f32];
        let screen_transform = calc_screen_transform(world_dim, window_dim);

        // drawing
        let mut frame = display.draw();
        frame.clear_color(0.0, 0.0, 0.0, 1.0);
        background_drawing.draw(&mut frame, &world_vb, screen_transform);
        bullseye_drawing.draw(&mut frame, &bullseyes_vb, screen_transform);
        frame.finish().unwrap();

        events_loop.poll_events(|event| {
            closed = is_window_close(&event);
        });
    }
}

fn calc_screen_transform(world_dim: [f32; 2], window_dim: [f32; 2]) -> [[f32; 4]; 4] {
    let x_scale;
    let y_scale;

    if world_dim[0] / world_dim[1] > window_dim[0] / window_dim[1] {
        x_scale = 2.0 / world_dim[0];
        y_scale = 2.0 / world_dim[0] * (window_dim[0] / window_dim[1]);
    } else {
        x_scale = 2.0 / world_dim[1] * (window_dim[1] / window_dim[0]);
        y_scale = 2.0 / world_dim[1];
    }

    [
        [x_scale, 0.0, 0.0, 0.0],
        [0.0, y_scale, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

fn is_window_close(event: &glutin::Event) -> bool {
    match event {
        glutin::Event::WindowEvent { event, .. } => match event {
            glutin::WindowEvent::CloseRequested => true,
            // Break from the loop upon `Escape`.
            glutin::WindowEvent::KeyboardInput {
                input: glutin::KeyboardInput {
                    virtual_keycode: Some(glutin::VirtualKeyCode::Escape),
                    ..
                },
                ..
            } => true,
            _ => false,
        },
        _ => false,
    }
}
