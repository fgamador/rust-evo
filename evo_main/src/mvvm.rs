use evo_conrod;
use evo_model;
use evo_model::physics::ball::Ball;
use evo_model::physics::quantities::*;
use evo_model::world::World;
use evo_view_model::ViewModel;
use std::thread;
use std::time::{Duration, Instant};

pub struct MVVM(pub Model, pub View, pub ViewModel);

pub struct Model {
    world: World,
}

impl Model {
    pub fn new() -> Self {
        let mut world = World::new(Position::new(-200.0, -200.0), Position::new(200.0, 200.0));
        world.add_ball(Ball::new(Length::new(20.0), Mass::new(1.0),
                                 Position::new(-100.0, -90.0), Velocity::new(3.0, 2.5)));
        world.add_ball(Ball::new(Length::new(20.0), Mass::new(1.0),
                                 Position::new(-90.0, 100.0), Velocity::new(2.5, -3.0)));
        world.add_ball(Ball::new(Length::new(20.0), Mass::new(1.0),
                                 Position::new(100.0, 90.0), Velocity::new(-3.0, -2.5)));
        world.add_ball(Ball::new(Length::new(20.0), Mass::new(1.0),
                                 Position::new(90.0, -100.0), Velocity::new(-2.5, 3.0)));
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
