#[macro_use]
extern crate glium;
extern crate evo_view_model;

use glium::{glutin, Surface};

pub mod background_drawing;
pub mod cell_drawing;

use background_drawing::*;
use cell_drawing::*;
use evo_view_model::ViewModel;

type Point = [f32; 2];

pub struct GliumView {
    events_loop: glutin::EventsLoop,
    display: glium::Display,
    world_min_corner: Point,
    world_max_corner: Point,
    background_drawing: BackgroundDrawing,
    cell_drawing: CellDrawing,
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
        let bullseye_drawing = CellDrawing::new(&display);
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
            cell_drawing: bullseye_drawing,
            world_vb,
        }
    }

    pub fn once(&mut self, view_model: &ViewModel) -> bool {
        if !self.handle_events() {
            return false;
        }

        self.draw_frame(&Self::view_model_bullseyes_to_drawing_cells(&view_model.bullseyes),
                        Self::get_layer_colors(&view_model.bullseyes));
        true
    }

    fn handle_events(&mut self) -> bool {
        let mut closed = false;
        self.events_loop.poll_events(|event| {
            closed = Self::is_window_close(&event);
        });
        !closed
    }

    fn view_model_bullseyes_to_drawing_cells(bullseyes: &[evo_view_model::Bullseye]) -> Vec<Cell> {
        bullseyes.iter().map(Self::view_model_bullseye_to_drawing_cell).collect()
    }

    fn view_model_bullseye_to_drawing_cell(bullseye: &evo_view_model::Bullseye) -> Cell {
        let mut radii: [f32; 8] = [0.0; 8];
        let mut health: [f32; 8] = [0.0; 8];
        assert!(bullseye.rings.len() <= radii.len());
        for (i, ring) in bullseye.rings.iter().enumerate() {
            radii[i] = ring.outer_radius as f32;
            health[i] = ring.health as f32;
        }
        Cell {
            center: [bullseye.center.x as f32, bullseye.center.y as f32],
            num_layers: bullseye.rings.len() as u32,
            radii_0_3: [radii[0], radii[1], radii[2], radii[3]],
            radii_4_7: [radii[4], radii[5], radii[6], radii[7]],
            health_0_3: [health[0], health[1], health[2], health[3]],
            health_4_7: [health[4], health[5], health[6], health[7]],
        }
    }

    fn get_layer_colors(bullseyes: &[evo_view_model::Bullseye]) -> [[f32; 4]; 8] {
        let mut layer_colors: [[f32; 4]; 8] = [[0.0, 0.0, 0.0, 1.0]; 8];
        if !bullseyes.is_empty() {
            let sample_cell = &bullseyes[0];
            assert!(sample_cell.rings.len() <= layer_colors.len());
            for (i, ring) in sample_cell.rings.iter().enumerate() {
                layer_colors[i] = Self::convert_to_rgb_color(ring.color);
            }
        }
        layer_colors
    }

    fn convert_to_rgb_color(color: evo_view_model::Color) -> [f32; 4] {
        match color {
            evo_view_model::Color::Green => [0.1, 0.8, 0.1, 1.0],
            evo_view_model::Color::White => [1.0, 1.0, 1.0, 1.0],
            evo_view_model::Color::Yellow => [0.7, 0.7, 0.0, 1.0],
        }
    }

    fn draw_frame(&mut self, cells: &Vec<Cell>, layer_colors: [[f32; 4]; 8]) {
        let cells_vb = glium::VertexBuffer::new(&self.display, &cells).unwrap();
        let screen_transform = self.current_screen_transform();
        let mut frame = self.display.draw();
        frame.clear_color(0.0, 0.0, 0.0, 1.0);
        self.background_drawing.draw(&mut frame, &self.world_vb, screen_transform);
        self.cell_drawing.draw(&mut frame, &cells_vb, screen_transform, layer_colors);
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
