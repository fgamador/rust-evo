extern crate evo_conrod;
extern crate evo_model;
extern crate evo_view_model;

use evo_view_model::Event;
use evo_view_model::ViewModel;
use evo_view_model::events::EventManager;

struct Model {}

struct View {}

struct MVVM(Model, View, ViewModel);

fn main() {
    let mut event_manager: EventManager<Event, MVVM> = EventManager::new();

    event_manager.add_listener(Event::Rendered, |event_queue, subject| {
        let MVVM(ref mut model, _, ref mut view_model) = subject;
        model.tick(view_model);
        event_queue.push(Event::Updated);
    });

    event_manager.add_listener(Event::Updated, |event_queue, subject| {
        let MVVM(_, ref mut view, ref mut view_model) = subject;
        view.render(view_model);
        event_queue.push(Event::Rendered);
    });

    let model = Model {};
    let view = View {};
    let view_model = ViewModel::new();
    let mut mvvm = MVVM(model, view, view_model);

    event_manager.events().push(Event::Rendered);
    event_manager.fire_events(&mut mvvm);

    //evo_conrod::main();
}

impl Model {
    pub fn tick(&mut self, view_model: &mut ViewModel) {
        evo_model::tick(view_model);
    }
}

impl View {
    pub fn render(&mut self, view_model: &mut ViewModel) {
        evo_conrod::render(view_model);
    }
}
