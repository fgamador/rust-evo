use crate::view::*;
use clap::{clap_app, ArgMatches};
use evo_domain::physics::quantities::Position;
use evo_domain::world::World;
use evo_domain::UserAction;
use std::thread;
use std::time::{Duration, Instant};

pub fn init_and_run(world: World) {
    let args = parse_command_line();
    let view = View::new(world.min_corner(), world.max_corner());
    run(world, view, args);
}

struct CommandLineArgs {
    _seed: u64,
    start_paused: bool,
}

fn parse_command_line() -> CommandLineArgs {
    let matches = clap_app!(evo =>
        (version: "0.1.0")
        (author: "Franz Amador <franzamador@gmail.com>")
        (about: "Evolution of simple digital organisms")
        (@arg seed: -s --seed +takes_value {is_u64_arg} "Random seed; defaults to 0")
        (@arg paused: -p --paused "Start with the simulation paused; press P to resume, T to run single tick")
    )
    .get_matches();

    CommandLineArgs {
        _seed: get_u64_arg(&matches, "seed", 0),
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

fn run(mut world: World, mut view: View, args: CommandLineArgs) {
    view.render(&world);

    let mut user_action = if args.start_paused {
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
