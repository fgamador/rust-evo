extern crate evo_conrod;
extern crate evo_model;
extern crate evo_view_model;

use evo_view_model::Event;
use evo_view_model::ViewModel;
use evo_view_model::events::EventManager;
use std::thread;
use std::time::{Duration, Instant};

struct Model {}

struct View {
    view: evo_conrod::feature::View,
    next_tick: Instant,
}

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
        if view.render(view_model) {
            event_queue.push(Event::Rendered);
        }
    });

    let mut mvvm = MVVM(Model::new(), View::new(), ViewModel::new());

    event_manager.events().push(Event::Rendered);
    event_manager.fire_events(&mut mvvm);
}

impl Model {
    pub fn new() -> Self {
        Model {}
    }

    pub fn tick(&mut self, view_model: &mut ViewModel) {
        evo_model::tick(view_model);
    }
}

impl View {
    pub fn new() -> Self {
        View {
            view: evo_conrod::feature::View::new(),
            next_tick: Instant::now(),
        }
    }

    pub fn render(&mut self, view_model: &mut ViewModel) -> bool {
        self.await_next_tick();
        self.view.once(view_model)
    }

    fn await_next_tick(&mut self) {
        let mut now = Instant::now();
        if now < self.next_tick {
            thread::sleep(self.next_tick - now);
            now = Instant::now();
        }
        self.next_tick = now + Duration::from_millis(16);
    }
}
