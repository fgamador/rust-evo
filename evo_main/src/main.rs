extern crate evo_conrod;
extern crate evo_model;
extern crate evo_view_model;

pub mod mvvm;

use evo_model::physics::ball::Ball;
use evo_model::physics::quantities::*;
use evo_model::world::World;
use evo_view_model::Event;
use evo_view_model::ViewModel;
use evo_view_model::events::EventManager;
use mvvm::*;

fn main() {
    let world = create_world();
    init_and_run(world);
}

fn init_and_run(world: World) {
    let mut event_manager: EventManager<Event, MVVM> = EventManager::new();
    wire_up_events(&mut event_manager);
    let mvvm = MVVM(Model::new(world), View::new(), ViewModel::new());
    run(event_manager, mvvm);
}

fn wire_up_events(event_manager: &mut EventManager<Event, MVVM>) {
    event_manager.add_listener(Event::Rendered, |event_queue, subject| {
        let MVVM(ref mut model, _, ref mut view_model) = subject;
        model.tick(view_model);
        event_queue.push(Event::Updated);
    });
    event_manager.add_listener(Event::Updated, |event_queue, subject| {
        let MVVM(_, ref mut view, ref mut view_model) = subject;
        if view.render(view_model) {
            event_queue.push(Event::Rendered);
        }
    });
}

fn run(mut event_manager: EventManager<Event, MVVM>, mut mvvm: MVVM) {
    event_manager.events().push(Event::Rendered);
    event_manager.fire_events(&mut mvvm);
}

fn create_world() -> World {
    let mut world = World::new(Position::new(-200.0, -200.0), Position::new(200.0, 200.0));
    world.add_ball(Ball::new(Length::new(20.0), Mass::new(1.0),
                             Position::new(-100.0, -90.0), Velocity::new(3.0, 2.5)));
    world.add_ball(Ball::new(Length::new(20.0), Mass::new(1.0),
                             Position::new(-90.0, 100.0), Velocity::new(2.5, -3.0)));
    world.add_ball(Ball::new(Length::new(20.0), Mass::new(1.0),
                             Position::new(100.0, 90.0), Velocity::new(-3.0, -2.5)));
    world.add_ball(Ball::new(Length::new(20.0), Mass::new(1.0),
                             Position::new(90.0, -100.0), Velocity::new(-2.5, 3.0)));
    world
}
