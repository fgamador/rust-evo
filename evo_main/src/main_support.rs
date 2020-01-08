use crate::view::*;
use evo_model::world::World;
use evo_model::UserAction;
use std::thread;
use std::time::{Duration, Instant};

pub fn init_and_run(world: World) {
    let view = View::new(world.min_corner(), world.max_corner());
    run(world, view);
}

fn run(mut world: World, mut view: View) {
    normal_speed(&mut world, &mut view);
}

fn normal_speed(world: &mut World, view: &mut View) -> UserAction {
    let mut next_tick = Instant::now();
    loop {
        if let Some(user_action) = view.check_for_user_action() {
            match user_action {
                UserAction::Exit | UserAction::PauseOrPlay => return user_action,
                _ => ()
            }
        }

        view.render(world);
        next_tick += Duration::from_millis(16);
        await_next_tick(next_tick);
        world.tick();
    }
}

fn await_next_tick(next_tick: Instant) {
    let now = Instant::now();
    if now < next_tick {
        thread::sleep(next_tick - now);
    }
}
