use evo_model::biology::layers::*;
use evo_model::environment::environment::*;
use evo_model::physics::newtonian::NewtonianBody;
use evo_model::physics::shapes::*;
use evo_model::physics::sortable_graph::*;
use evo_model::world::World;
use evo_view_model::Event;
use evo_view_model::ViewModel;
use evo_view_model::events::EventManager;
use mvvm::*;

pub fn init_and_run<T>(world: World<T>)
    where T: Circle + GraphNode + HasLocalEnvironment + NewtonianBody + Onion
{
    let mut event_manager: EventManager<Event, MVVM<T>> = EventManager::new();
    wire_up_events(&mut event_manager);
    let view = View::new(world.min_corner(), world.max_corner());
    let mvvm = MVVM(Model::new(world), view, ViewModel::new());
    run(event_manager, mvvm);
}

fn wire_up_events<T>(event_manager: &mut EventManager<Event, MVVM<T>>)
    where T: Circle + GraphNode + HasLocalEnvironment + NewtonianBody + Onion
{
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

fn run<T>(mut event_manager: EventManager<Event, MVVM<T>>, mut mvvm: MVVM<T>)
    where T: Circle + GraphNode + HasLocalEnvironment + NewtonianBody + Onion
{
    event_manager.events().push(Event::Rendered);
    event_manager.fire_events(&mut mvvm);
}
