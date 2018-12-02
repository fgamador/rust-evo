use evo_conrod::feature::ConrodView;
use evo_model;
use evo_model::TickCallbacks;
use evo_model::biology::layers::*;
use evo_model::environment::environment::*;
use evo_model::physics::newtonian::NewtonianBody;
use evo_model::physics::quantities::*;
use evo_model::physics::shapes::*;
use evo_model::physics::sortable_graph::*;
use evo_model::world::World;
use evo_view_model;
use evo_view_model::ViewModel;
use evo_view_model::CoordinateTransform;
use std::thread;
use std::time::{Duration, Instant};

pub struct MVVM<C>(pub Model<C>, pub View, pub ViewModel)
    where C: Circle + GraphNode + HasLocalEnvironment + NewtonianBody + Onion + TickCallbacks;

pub struct Model<C>
    where C: Circle + GraphNode + HasLocalEnvironment + NewtonianBody + Onion + TickCallbacks
{
    world: World<C>,
}

impl<C> Model<C>
    where C: Circle + GraphNode + HasLocalEnvironment + NewtonianBody + Onion + TickCallbacks
{
    pub fn new(world: World<C>) -> Self {
        Model {
            world
        }
    }

    pub fn tick(&mut self, view_model: &mut ViewModel) {
        evo_model::tick(&mut self.world, view_model);
    }
}

pub struct View {
    view: ConrodView,
    next_tick: Instant,
}

impl View {
    pub fn new(world_min_corner: Position, world_max_corner: Position) -> Self {
        let transform = Self::create_coordinate_transform(world_min_corner, world_max_corner);
        View {
            view: ConrodView::new(transform),
            next_tick: Instant::now(),
        }
    }

    fn create_coordinate_transform(input_min_corner: Position, input_max_corner: Position) -> CoordinateTransform {
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

    pub fn render(&mut self, view_model: &mut ViewModel) -> bool {
        self.await_next_tick();
        self.view.once(view_model)
    }

    fn await_next_tick(&mut self) {
        let now = Instant::now();
        if now < self.next_tick {
            thread::sleep(self.next_tick - now);
        }
        self.next_tick += Duration::from_millis(16);
    }
}
