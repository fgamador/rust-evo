use evo_glium::GliumView;
use evo_model::physics::quantities::*;
use evo_model::world::World;
use std::thread;
use std::time::{Duration, Instant};

pub struct View {
    view: GliumView,
    next_tick: Instant,
}

impl View {
    pub fn new(world_min_corner: Position, world_max_corner: Position) -> Self {
        View {
            view: GliumView::new(
                [world_min_corner.x() as f32, world_min_corner.y() as f32],
                [world_max_corner.x() as f32, world_max_corner.y() as f32],
            ),
            next_tick: Instant::now(),
        }
    }

    pub fn render(&mut self, world: &World) -> bool {
        self.await_next_tick();
        self.view.once(world)
    }

    fn await_next_tick(&mut self) {
        let now = Instant::now();
        if now < self.next_tick {
            thread::sleep(self.next_tick - now);
        }
        self.next_tick += Duration::from_millis(16);
    }
}
