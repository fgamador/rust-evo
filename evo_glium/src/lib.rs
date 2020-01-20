#[macro_use]
extern crate glium;
extern crate evo_model;

use glium::{glutin, Surface};

pub mod background_drawing;
pub mod cell_drawing;

use background_drawing::*;
use cell_drawing::*;
use evo_model::biology::layers;
use evo_model::biology::layers::Onion;
use evo_model::biology::layers::OnionLayer;
use evo_model::physics::shapes::Circle;
use evo_model::UserAction;

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
        let window = glutin::WindowBuilder::new().with_dimensions(Self::calc_initial_window_size(
            world_min_corner,
            world_max_corner,
        ));
        let context = glutin::ContextBuilder::new()
            .with_vsync(true)
            .with_multisampling(4);
        let events_loop = glutin::EventsLoop::new();
        let display = glium::Display::new(window, context, &events_loop).unwrap();
        let background_drawing = BackgroundDrawing::new(&display);
        let bullseye_drawing = CellDrawing::new(&display);
        let world = vec![World {
            corners: [
                world_min_corner[0],
                world_min_corner[1],
                world_max_corner[0],
                world_max_corner[1],
            ],
            top_color: [0.0, 0.1, 0.5],
            bottom_color: [0.0, 0.0, 0.0],
        }];
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

    fn calc_initial_window_size(
        world_min_corner: Point,
        world_max_corner: Point,
    ) -> glutin::dpi::LogicalSize {
        //        let world_width = world_max_corner[0] - world_min_corner[0];
        //        let world_height = world_max_corner[1] - world_min_corner[1];
        glutin::dpi::LogicalSize::new(500.0, 500.0)
    }

    pub fn render(&mut self, world: &evo_model::world::World) {
        self.draw_frame(
            &Self::view_model_bullseyes_to_drawing_cells(world),
            Self::get_layer_colors(world),
        );
    }

    fn view_model_bullseyes_to_drawing_cells(world: &evo_model::world::World) -> Vec<Cell> {
        world
            .cells()
            .iter()
            .map(Self::model_cell_to_drawing_cell)
            .collect()
    }

    fn model_cell_to_drawing_cell(cell: &evo_model::biology::cell::Cell) -> Cell {
        let mut radii: [f32; 8] = [0.0; 8];
        let mut health: [f32; 8] = [0.0; 8];
        assert!(cell.layers().len() <= radii.len());
        for (i, layer) in cell.layers().iter().enumerate() {
            radii[i] = layer.outer_radius().value() as f32;
            health[i] = layer.health() as f32;
        }
        Cell {
            center: [cell.center().x() as f32, cell.center().y() as f32],
            num_layers: cell.layers().len() as u32,
            radii_0_3: [radii[0], radii[1], radii[2], radii[3]],
            radii_4_7: [radii[4], radii[5], radii[6], radii[7]],
            health_0_3: [health[0], health[1], health[2], health[3]],
            health_4_7: [health[4], health[5], health[6], health[7]],
        }
    }

    fn get_layer_colors(world: &evo_model::world::World) -> [[f32; 4]; 8] {
        let mut layer_colors: [[f32; 4]; 8] = [[0.0, 0.0, 0.0, 1.0]; 8];
        if !world.cells().is_empty() {
            let sample_cell = &world.cells()[0];
            assert!(sample_cell.layers().len() <= layer_colors.len());
            for (i, layer) in sample_cell.layers().iter().enumerate() {
                layer_colors[i] = Self::convert_to_rgb_color(layer.color());
            }
        }
        layer_colors
    }

    fn convert_to_rgb_color(color: layers::Color) -> [f32; 4] {
        match color {
            layers::Color::Green => [0.1, 0.8, 0.1, 1.0],
            layers::Color::White => [1.0, 1.0, 1.0, 1.0],
            layers::Color::Yellow => [0.7, 0.7, 0.0, 1.0],
        }
    }

    fn draw_frame(&mut self, cells: &[Cell], layer_colors: [[f32; 4]; 8]) {
        let cells_vb = glium::VertexBuffer::new(&self.display, &cells).unwrap();
        let screen_transform = self.current_screen_transform();
        let mut frame = self.display.draw();
        frame.clear_color(0.0, 0.0, 0.0, 1.0);
        self.background_drawing
            .draw(&mut frame, &self.world_vb, screen_transform);
        self.cell_drawing
            .draw(&mut frame, &cells_vb, screen_transform, layer_colors);
        frame.finish().unwrap();
    }

    fn current_screen_transform(&mut self) -> [[f32; 4]; 4] {
        // TODO more efficient to do this only on glutin::WindowEvent::Resized
        let window_size = self.display.gl_window().window().get_inner_size().unwrap();
        let window_dim = [window_size.width as f32, window_size.height as f32];
        Self::calc_screen_transform(self.world_min_corner, self.world_max_corner, window_dim)
    }

    #[allow(clippy::useless_let_if_seq)]
    fn calc_screen_transform(
        world_min_corner: Point,
        world_max_corner: Point,
        window_dim: [f32; 2],
    ) -> [[f32; 4]; 4] {
        let world_dim = [
            world_max_corner[0] - world_min_corner[0],
            world_max_corner[1] - world_min_corner[1],
        ];

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

    pub fn check_for_user_action(&mut self) -> Option<UserAction> {
        let mut result = None;
        self.events_loop.poll_events(|event| {
            // drain the event queue, capturing the first user action
            if result == None {
                result = Self::interpret_event_as_user_action(&event);
            }
        });
        result
    }

    pub fn wait_for_user_action(&mut self) -> UserAction {
        let mut result = UserAction::Exit; // bogus initial value
        self.events_loop
            .run_forever(|event| -> glutin::ControlFlow {
                if let Some(user_action) = Self::interpret_event_as_user_action(&event) {
                    result = user_action;
                    glutin::ControlFlow::Break
                } else {
                    glutin::ControlFlow::Continue
                }
            });
        result
    }

    fn interpret_event_as_user_action(event: &glutin::Event) -> Option<UserAction> {
        match event {
            glutin::Event::WindowEvent { event, .. } => match event {
                glutin::WindowEvent::CloseRequested => Some(UserAction::Exit),
                glutin::WindowEvent::KeyboardInput {
                    input:
                        glutin::KeyboardInput {
                            state: glutin::ElementState::Pressed,
                            virtual_keycode: Some(key_code),
                            ..
                        },
                    ..
                } => Self::interpret_key_as_user_action(*key_code),
                _ => None,
            },
            _ => None,
        }
    }

    fn interpret_key_as_user_action(key_code: glutin::VirtualKeyCode) -> Option<UserAction> {
        match key_code {
            glutin::VirtualKeyCode::D => Some(UserAction::DebugPrint),
            glutin::VirtualKeyCode::Escape
            | glutin::VirtualKeyCode::Q
            | glutin::VirtualKeyCode::X => Some(UserAction::Exit),
            glutin::VirtualKeyCode::P => Some(UserAction::PlayToggle),
            glutin::VirtualKeyCode::S => Some(UserAction::SingleTick),
            _ => None,
        }
    }
}
