use evo_conrod;
use evo_model;
use evo_model::environment::environment::*;
use evo_model::physics::newtonian::NewtonianBody;
use evo_model::physics::shapes::Circle;
use evo_model::physics::sortable_graph::*;
use evo_model::world::World;
use evo_view_model::ViewModel;
use std::thread;
use std::time::{Duration, Instant};

pub struct MVVM<T>(pub Model<T>, pub View, pub ViewModel)
    where T: Circle + GraphNode + NewtonianBody + HasLocalEnvironment;

pub struct Model<T>
    where T: Circle + GraphNode + NewtonianBody + HasLocalEnvironment
{
    world: World<T>,
}

impl<T> Model<T>
    where T: Circle + GraphNode + NewtonianBody + HasLocalEnvironment
{
    pub fn new(world: World<T>) -> Self {
        Model {
            world
        }
    }

    pub fn tick(&mut self, view_model: &mut ViewModel) {
        evo_model::tick(&mut self.world, view_model);
    }
}

pub struct View {
    view: evo_conrod::feature::View,
    next_tick: Instant,
}

impl View {
    pub fn new() -> Self {
        View {
            view: evo_conrod::feature::View::new(),
            next_tick: Instant::now(),
        }
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
