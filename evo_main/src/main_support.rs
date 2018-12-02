use evo_model::TickCallbacks;
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

pub fn init_and_run<C>(world: World<C>)
    where C: Circle + GraphNode + HasLocalEnvironment + NewtonianBody + Onion + TickCallbacks
{
    let mut event_manager: EventManager<Event, MVVM<C>> = EventManager::new();
    wire_up_events(&mut event_manager);
    let view = View::new(world.min_corner(), world.max_corner());
    let mvvm = MVVM(Model::new(world), view, ViewModel::new());
    run(event_manager, mvvm);
}

fn wire_up_events<C>(event_manager: &mut EventManager<Event, MVVM<C>>)
    where C: Circle + GraphNode + HasLocalEnvironment + NewtonianBody + Onion + TickCallbacks
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

fn run<C>(mut event_manager: EventManager<Event, MVVM<C>>, mut mvvm: MVVM<C>)
    where C: Circle + GraphNode + HasLocalEnvironment + NewtonianBody + Onion + TickCallbacks
{
    event_manager.events().push(Event::Rendered);
    event_manager.fire_events(&mut mvvm);
}
