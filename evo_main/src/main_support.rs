use crate::view::*;
use clap::{clap_app, ArgMatches};
use evo_domain::physics::quantities::Position;
use evo_domain::world::World;
use evo_domain::UserAction;
use std::thread;
use std::time::{Duration, Instant};

pub fn init_and_run<F>(create_world: F)
where
    F: Fn(u64) -> World,
{
    let args = parse_command_line();
    let world = create_world(args.seed);
    let view = View::new(world.min_corner(), world.max_corner());
    run(world, view, args.start_paused);
}

pub struct CommandLineArgs {
    pub seed: u64,
    pub start_paused: bool,
}

pub fn parse_command_line() -> CommandLineArgs {
    let matches = clap_app!(evo =>
        (version: "0.1.0")
        (author: "Franz Amador <franzamador@gmail.com>")
        (about: "Evolution of simple digital organisms")
        (@arg seed: -s --seed +takes_value {is_u64_arg} "Random seed, defaults to 0")
        (@arg paused: -p --paused "Start with the simulation paused. Press P to resume, T to run single tick.")
    )
    .get_matches();

    CommandLineArgs {
        seed: get_u64_arg(&matches, "seed", 0),
        start_paused: matches.is_present("paused"),
    }
}

fn is_u64_arg(v: String) -> Result<(), String> {
    if v.parse::<u64>().is_ok() {
        Ok(())
    } else {
        Err("Not a positive integer".to_string())
    }
}

fn get_u64_arg(matches: &ArgMatches, name: &str, default_value: u64) -> u64 {
    if let Some(arg) = matches.value_of(name) {
        arg.parse::<u64>().unwrap()
    } else {
        default_value
    }
}

fn run(mut world: World, mut view: View, start_paused: bool) {
    view.render(&world);

    let mut next_action = if start_paused {
        Some(view.wait_for_user_action())
    } else {
        Some(UserAction::PlayToggle)
    };

    while let Some(this_action) = next_action {
        next_action = match this_action {
            UserAction::DebugPrint => Some(debug_print_action(&mut world, &mut view)),

            UserAction::Exit => None,

            UserAction::FastForwardToggle => Some(fast_forward_action(&mut world, &mut view)),

            UserAction::PlayToggle => Some(play_action(&mut world, &mut view)),

            UserAction::SelectCellToggle { x, y } => {
                Some(toggle_select_cell_action(&mut world, &mut view, x, y))
            }

            UserAction::SingleTick => Some(single_tick_action(&mut world, &mut view)),
        };
    }
}

fn debug_print_action(world: &mut World, view: &mut View) -> UserAction {
    world.debug_print_cells();
    view.wait_for_user_action()
}

fn fast_forward_action(world: &mut World, view: &mut View) -> UserAction {
    let action = fast_forward(world, view);
    if action == UserAction::FastForwardToggle {
        view.wait_for_user_action()
    } else {
        action
    }
}

fn fast_forward(world: &mut World, view: &mut View) -> UserAction {
    loop {
        if let Some(user_action) = view.check_for_user_action() {
            return user_action;
        }

        tick_for(world, Duration::from_millis(16));
        view.render(world);
    }
}

fn tick_for(world: &mut World, duration: Duration) {
    let end_time = Instant::now() + duration;
    while Instant::now() < end_time {
        world.tick();
    }
}

fn play_action(world: &mut World, view: &mut View) -> UserAction {
    let action = play(world, view);
    if action == UserAction::PlayToggle {
        view.wait_for_user_action()
    } else {
        action
    }
}

fn play(world: &mut World, view: &mut View) -> UserAction {
    let mut next_tick = Instant::now();
    loop {
        if let Some(user_action) = view.check_for_user_action() {
            return user_action;
        }

        single_tick(world, view);

        next_tick += Duration::from_millis(16);
        await_next_tick(next_tick);
    }
}

fn await_next_tick(next_tick: Instant) {
    let now = Instant::now();
    if now < next_tick {
        thread::sleep(next_tick - now);
    }
}

fn toggle_select_cell_action(world: &mut World, view: &mut View, x: f64, y: f64) -> UserAction {
    world.toggle_select_cell_at(Position::new(x, y));
    view.render(world);
    view.wait_for_user_action()
}

fn single_tick_action(world: &mut World, view: &mut View) -> UserAction {
    single_tick(world, view);
    view.wait_for_user_action()
}

fn single_tick(world: &mut World, view: &mut View) {
    world.tick();
    view.render(world);
}
