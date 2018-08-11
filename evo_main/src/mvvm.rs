use evo_conrod;
use evo_model;
use evo_model::world::World;
use evo_view_model::ViewModel;
use std::thread;
use std::time::{Duration, Instant};

pub struct MVVM(pub Model, pub View, pub ViewModel);

pub struct Model {
    world: World,
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
