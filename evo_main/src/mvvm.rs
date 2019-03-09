//use evo_conrod::feature::ConrodView;
use evo_glium::GliumView;
use evo_model;
use evo_model::physics::quantities::*;
use evo_model::world::World;
use evo_view_model;
use evo_view_model::ViewModel;
use evo_view_model::CoordinateTransform;
use std::thread;
use std::time::{Duration, Instant};

pub struct MVVM(pub Model, pub View, pub ViewModel);

pub struct Model {
    pub world: World,
}

impl Model {
    pub fn new(world: World) -> Self {
        Model {
            world
        }
    }

    pub fn tick(&mut self, view_model: &mut ViewModel) {
        evo_model::tick(&mut self.world, view_model);
    }
}

pub struct View {
    //view: ConrodView,
    view: GliumView,
    next_tick: Instant,
}

impl View {
    pub fn new(world_min_corner: Position, world_max_corner: Position) -> Self {
//        let transform = Self::_create_coordinate_transform(world_min_corner, world_max_corner);
        View {
//            view: ConrodView::new(transform),
            view: GliumView::new(
                [world_min_corner.x() as f32, world_min_corner.y() as f32],
                [world_max_corner.x() as f32, world_max_corner.y() as f32]),
            next_tick: Instant::now(),
        }
    }

    fn _create_coordinate_transform(input_min_corner: Position, input_max_corner: Position) -> CoordinateTransform {
        let input_window = evo_view_model::Rectangle {
            min_corner: evo_view_model::Point {
                x: input_min_corner.x(),
                y: input_min_corner.y(),
            },
            max_corner: evo_view_model::Point {
                x: input_max_corner.x(),
                y: input_max_corner.y(),
            },
        };
        CoordinateTransform::new(input_window)
    }

    pub fn render(&mut self, world: &World, view_model: &mut ViewModel) -> bool {
        self.await_next_tick();
        self.view.once(world, view_model)
    }

    fn await_next_tick(&mut self) {
        let now = Instant::now();
        if now < self.next_tick {
            thread::sleep(self.next_tick - now);
        }
        self.next_tick += Duration::from_millis(16);
    }
}
