#[macro_use]
extern crate glium;
extern crate evo_view_model;

use glium::{glutin, Surface};

pub mod background_drawing;
pub mod bullseye_drawing;

use background_drawing::*;
use bullseye_drawing::*;
use evo_view_model::ViewModel;

type Point = [f32; 2];

pub struct GliumView {
    events_loop: glutin::EventsLoop,
    display: glium::Display,
    world_min_corner: Point,
    world_max_corner: Point,
    background_drawing: BackgroundDrawing,
    bullseye_drawing: BullseyeDrawing,
    world_vb: glium::VertexBuffer<World>,
}

impl GliumView {
    pub fn new(world_min_corner: Point, world_max_corner: Point) -> Self {
        let events_loop = glutin::EventsLoop::new();
        let window = glutin::WindowBuilder::new()
            .with_dimensions(glutin::dpi::LogicalSize::new(500.0, 500.0));
        let context = glutin::ContextBuilder::new();
        let display = glium::Display::new(window, context, &events_loop).unwrap();
        let background_drawing = BackgroundDrawing::new(&display);
        let bullseye_drawing = BullseyeDrawing::new(&display);
        let world = vec![
            World {
                corners: [world_min_corner[0], world_min_corner[1], world_max_corner[0], world_max_corner[1]],
                top_color: [0.0, 0.1, 0.5],
                bottom_color: [0.0, 0.0, 0.0],
            }
        ];
        let world_vb = glium::VertexBuffer::new(&display, &world).unwrap();

        GliumView {
            events_loop,
            display,
            world_min_corner,
            world_max_corner,
            background_drawing,
            bullseye_drawing,
            world_vb,
        }
    }

    pub fn once(&mut self, view_model: &ViewModel) -> bool {
        if !self.handle_events() {
            return false;
        }

        self.draw_frame(&Self::view_model_bullseyes_to_drawing_bullseyes(&view_model.bullseyes));
        true
    }

    fn handle_events(&mut self) -> bool {
        let mut closed = false;
        self.events_loop.poll_events(|event| {
            closed = Self::is_window_close(&event);
        });
        !closed
    }

    fn view_model_bullseyes_to_drawing_bullseyes(bullseyes: &[evo_view_model::Bullseye]) -> Vec<Bullseye> {
        bullseyes.iter().map(Self::view_model_bullseye_to_drawing_bullseye).collect()
    }

    fn view_model_bullseye_to_drawing_bullseye(bullseye: &evo_view_model::Bullseye) -> Bullseye {
        let mut radii = [0.0_f32; 4];
        for (i, ring) in bullseye.rings.iter().enumerate() {
            radii[i] = ring.outer_radius as f32;
        }
        Bullseye {
            center: [bullseye.center.x as f32, bullseye.center.y as f32],
            radii_0_3: radii,
            num_radii: bullseye.rings.len() as u32,
        }
    }

    fn draw_frame(&mut self, bullseyes: &Vec<Bullseye>) {
        let bullseyes_vb = glium::VertexBuffer::new(&self.display, &bullseyes).unwrap();
        let screen_transform = self.current_screen_transform();
        let mut frame = self.display.draw();
        frame.clear_color(0.0, 0.0, 0.0, 1.0);
        self.background_drawing.draw(&mut frame, &self.world_vb, screen_transform);
        self.bullseye_drawing.draw(&mut frame, &bullseyes_vb, screen_transform);
        frame.finish().unwrap();
    }

    fn current_screen_transform(&mut self) -> [[f32; 4]; 4] {
        // TODO more efficient to do this only on glutin::WindowEvent::Resized
        let window_size = self.display.gl_window().get_inner_size().unwrap();
        let window_dim = [window_size.width as f32, window_size.height as f32];
        Self::calc_screen_transform(self.world_min_corner, self.world_max_corner, window_dim)
    }

    fn calc_screen_transform(world_min_corner: Point, world_max_corner: Point, window_dim: [f32; 2]) -> [[f32; 4]; 4] {
        let world_dim = [world_max_corner[0] - world_min_corner[0], world_max_corner[1] - world_min_corner[1]];

        let x_scale;
        let y_scale;

        if world_dim[0] / world_dim[1] > window_dim[0] / window_dim[1] {
            x_scale = 2.0 / world_dim[0];
            y_scale = 2.0 / world_dim[0] * (window_dim[0] / window_dim[1]);
        } else {
            x_scale = 2.0 / world_dim[1] * (window_dim[1] / window_dim[0]);
            y_scale = 2.0 / world_dim[1];
        }

        let x_midpoint = world_min_corner[0] + world_dim[0] / 2.0;
        let y_midpoint = world_min_corner[1] + world_dim[1] / 2.0;
        let x_translate = -x_scale * x_midpoint;
        let y_translate = -y_scale * y_midpoint;

        [
            [x_scale, 0.0, 0.0, 0.0],
            [0.0, y_scale, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [x_translate, y_translate, 0.0, 1.0],
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
}
