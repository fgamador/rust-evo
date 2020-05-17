use glium::{glutin, Surface};

mod background_drawing;
mod cell_drawing;

use background_drawing::*;
use cell_drawing::*;
use evo_model::biology::cell::Cell;
use evo_model::biology::layers;
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
    mouse_position: glutin::dpi::LogicalPosition,
}

impl GliumView {
    pub fn new(world_min_corner: Point, world_max_corner: Point) -> Self {
        let events_loop = glutin::EventsLoop::new();
        let window = glutin::WindowBuilder::new().with_dimensions(Self::calc_initial_window_size(
            (
                (world_max_corner[0] - world_min_corner[0]) as f64,
                (world_max_corner[1] - world_min_corner[1]) as f64,
            ),
            Self::get_screen_size(events_loop.get_primary_monitor()),
            0.75,
        ));
        let context = glutin::ContextBuilder::new()
            .with_vsync(true)
            .with_multisampling(4);
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
            mouse_position: glutin::dpi::LogicalPosition::new(0.0, 0.0),
        }
    }

    fn get_screen_size(monitor: glutin::MonitorId) -> glutin::dpi::LogicalSize {
        monitor
            .get_dimensions()
            .to_logical(monitor.get_hidpi_factor())
    }

    fn calc_initial_window_size(
        world_size: (f64, f64),
        screen_size: glutin::dpi::LogicalSize,
        desired_fraction_of_screen_dimension: f64,
    ) -> glutin::dpi::LogicalSize {
        let desired_window_width = desired_fraction_of_screen_dimension * screen_size.width;
        let desired_window_height = desired_fraction_of_screen_dimension * screen_size.height;
        let window_aspect_ratio = world_size.0 / world_size.1;
        if window_aspect_ratio > desired_window_width / desired_window_height {
            glutin::dpi::LogicalSize::new(
                desired_window_width,
                desired_window_width / window_aspect_ratio,
            )
        } else {
            glutin::dpi::LogicalSize::new(
                desired_window_height * window_aspect_ratio,
                desired_window_height,
            )
        }
    }

    pub fn render(&mut self, world: &evo_model::world::World) {
        self.draw_frame(
            &Self::world_cells_to_cell_sprites(world),
            Self::get_layer_colors(world),
        );
    }

    fn world_cells_to_cell_sprites(world: &evo_model::world::World) -> Vec<CellSprite> {
        world
            .cells()
            .iter()
            .map(Self::world_cell_to_cell_sprite)
            .collect()
    }

    fn world_cell_to_cell_sprite(cell: &Cell) -> CellSprite {
        let mut num_layers = cell.layers().len();
        let mut radii: [f32; 8] = [0.0; 8];
        let mut health: [f32; 8] = [0.0; 8];
        assert!(num_layers <= radii.len());
        for (i, layer) in cell.layers().iter().enumerate() {
            radii[i] = layer.outer_radius().value() as f32;
            health[i] = layer.health() as f32;
        }
        if cell.is_selected() {
            num_layers += 1;
            assert!(num_layers <= radii.len());
            radii[num_layers - 1] = (cell.radius().value() + 1.0) as f32;
            health[num_layers - 1] = 1.0;
        }
        CellSprite {
            center: [cell.center().x() as f32, cell.center().y() as f32],
            num_layers: num_layers as u32,
            radii_0_3: [radii[0], radii[1], radii[2], radii[3]],
            radii_4_7: [radii[4], radii[5], radii[6], radii[7]],
            health_0_3: [health[0], health[1], health[2], health[3]],
            health_4_7: [health[4], health[5], health[6], health[7]],
        }
    }

    fn get_layer_colors(world: &evo_model::world::World) -> [[f32; 4]; 8] {
        const SELECTION_HALO_COLOR: [f32; 4] = [1.0, 0.0, 0.2, 1.0];

        let mut layer_colors: [[f32; 4]; 8] = [[0.0, 0.0, 0.0, 1.0]; 8];
        if !world.cells().is_empty() {
            let sample_cell = &world.cells()[0];
            assert!(sample_cell.layers().len() < layer_colors.len());
            for (i, layer) in sample_cell.layers().iter().enumerate() {
                layer_colors[i] = Self::convert_to_rgb_color(layer.color());
            }
            layer_colors[sample_cell.layers().len()] = SELECTION_HALO_COLOR;
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

    fn draw_frame(&mut self, cells: &[CellSprite], layer_colors: [[f32; 4]; 8]) {
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
        let mut mouse_position = self.mouse_position;
        self.events_loop.poll_events(|event| {
            // drain the event queue, capturing the first user action
            if result == None {
                result = Self::interpret_event_as_user_action(&event, &mut mouse_position);
            }
        });
        self.mouse_position = mouse_position;
        result
    }

    pub fn wait_for_user_action(&mut self) -> UserAction {
        let mut result = UserAction::Exit; // bogus initial value
        let mut mouse_position = self.mouse_position;
        self.events_loop
            .run_forever(|event| -> glutin::ControlFlow {
                if let Some(user_action) =
                    Self::interpret_event_as_user_action(&event, &mut mouse_position)
                {
                    result = user_action;
                    glutin::ControlFlow::Break
                } else {
                    glutin::ControlFlow::Continue
                }
            });
        self.mouse_position = mouse_position;
        result
    }

    fn interpret_event_as_user_action(
        event: &glutin::Event,
        mouse_position: &mut glutin::dpi::LogicalPosition,
    ) -> Option<UserAction> {
        match event {
            glutin::Event::WindowEvent { event, .. } => match event {
                glutin::WindowEvent::CloseRequested => Some(UserAction::Exit),

                glutin::WindowEvent::CursorMoved { position, .. } => {
                    *mouse_position = *position;
                    None
                }

                glutin::WindowEvent::KeyboardInput {
                    input:
                        glutin::KeyboardInput {
                            state: glutin::ElementState::Pressed,
                            virtual_keycode: Some(key_code),
                            ..
                        },
                    ..
                } => Self::interpret_key_as_user_action(*key_code),

                glutin::WindowEvent::MouseInput {
                    button: glutin::MouseButton::Left,
                    state: glutin::ElementState::Pressed,
                    ..
                } => Some(UserAction::SelectCell {
                    x: mouse_position.x,
                    y: mouse_position.y,
                }),

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_window_size_for_world_wider_than_screen() {
        let initial_size = GliumView::calc_initial_window_size(
            (200.0, 100.0),
            glutin::dpi::LogicalSize::new(1000.0, 1000.0),
            0.5,
        );
        assert_eq!(initial_size, glutin::dpi::LogicalSize::new(500.0, 250.0));
    }

    #[test]
    fn initial_window_size_for_world_taller_than_screen() {
        let initial_size = GliumView::calc_initial_window_size(
            (100.0, 200.0),
            glutin::dpi::LogicalSize::new(1000.0, 1000.0),
            0.5,
        );
        assert_eq!(initial_size, glutin::dpi::LogicalSize::new(250.0, 500.0));
    }
}
