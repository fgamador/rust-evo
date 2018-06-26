extern crate evo_conrod;
extern crate evo_model;
extern crate evo_view_model;

use evo_view_model::Event;
use evo_view_model::ViewModel;
use evo_view_model::events::EventManager;

fn main() {
    let mut event_manager: EventManager<Event, TheWorks> = EventManager::new();

    event_manager.add_listener(Event::Rendered, |event_queue, subject| {
        let TheWorks(ref mut model, _, ref mut view_model) = subject;
        model.updated = true;
        view_model.update();
        event_queue.push(Event::Updated);
    });

    event_manager.add_listener(Event::Updated, |event_queue, subject| {
        let TheWorks(_, ref mut view, ref mut view_model) = subject;
        view.rendered = true;
        view_model.render();
        event_queue.push(Event::Rendered);
    });

    let model = Model { updated: false };
    let view = View { rendered: false };
    let view_model = ViewModel { updated: false };
    let mut the_works = TheWorks(model, view, view_model);

    event_manager.events().push(Event::Rendered);
    event_manager.fire_events(&mut the_works);

    //evo_conrod::main();
}

struct Model {
    updated: bool,
}

struct View {
    rendered: bool,
}

struct TheWorks(Model, View, ViewModel);
