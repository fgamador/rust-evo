extern crate evo_conrod;
extern crate evo_model;
extern crate evo_view_model;

pub mod mvvm;

use evo_view_model::Event;
use evo_view_model::ViewModel;
use evo_view_model::events::EventManager;
use mvvm::*;

fn main() {
    let mut event_manager: EventManager<Event, MVVM> = EventManager::new();

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

    let mut mvvm = MVVM(Model::new(), View::new(), ViewModel::new());

    event_manager.events().push(Event::Rendered);
    event_manager.fire_events(&mut mvvm);
}
