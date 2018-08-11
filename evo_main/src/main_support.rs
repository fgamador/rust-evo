use evo_model::world::World;
use evo_view_model::Event;
use evo_view_model::ViewModel;
use evo_view_model::events::EventManager;
use mvvm::*;

pub fn init_and_run(world: World) {
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
