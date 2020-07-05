use crate::view::*;
use evo_domain::physics::quantities::Position;
use evo_domain::world::World;
use evo_domain::UserAction;
use std::env;
use std::thread;
use std::time::{Duration, Instant};

pub fn init_and_run(world: World) {
    simple_logger::init().unwrap();

    let args: Vec<String> = env::args().collect();
    let start_paused = args.len() == 2 && args[1] == "-p".to_string();

    let view = View::new(world.min_corner(), world.max_corner());
    run(world, view, start_paused);
}

fn run(mut world: World, mut view: View, start_paused: bool) {
    view.render(&world);

    let mut user_action = if start_paused {
        UserAction::None
    } else {
        UserAction::PlayToggle
    };

    loop {
        match user_action {
            UserAction::DebugPrint => world.debug_print_cells(),
            UserAction::Exit => return,
            UserAction::None => (),
            UserAction::PlayToggle => {
                if normal_speed(&mut world, &mut view) == UserAction::Exit {
                    return;
                }
            }
            UserAction::SelectCellToggle { x, y } => {
                world.toggle_select_cell_at(Position::new(x, y));
                view.render(&world);
            }
            UserAction::SingleTick => single_tick(&mut world, &mut view),
        }
        user_action = view.wait_for_user_action();
    }
}

fn normal_speed(world: &mut World, view: &mut View) -> UserAction {
    let mut next_tick = Instant::now();
    loop {
        next_tick += Duration::from_millis(16);
        await_next_tick(next_tick);

        if let Some(user_action) = view.check_for_user_action() {
            if let UserAction::Exit | UserAction::PlayToggle = user_action {
                return user_action;
            }
        }

        single_tick(world, view);
    }
}

fn single_tick(world: &mut World, view: &mut View) {
    world.tick();
    view.render(world);
}

fn await_next_tick(next_tick: Instant) {
    let now = Instant::now();
    if now < next_tick {
        thread::sleep(next_tick - now);
    }
}
