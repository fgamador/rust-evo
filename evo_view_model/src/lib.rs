use std::rc::Rc;
use std::collections::HashMap;

type BoxedCallback = Rc<Fn(&mut ViewModel) -> ()>;

#[derive(Clone, PartialEq, Eq, Hash)]
enum Event {
    Rendered,
    Updated,
}

pub struct ViewModel {
    pub updated: bool,
    pub rendered: bool,
    events: Vec<Event>,
    listeners: HashMap<Event, Vec<BoxedCallback>>,
}

impl ViewModel {
    pub fn new() -> ViewModel {
        ViewModel {
            updated: false,
            rendered: false,
            events: Vec::new(),
            listeners: HashMap::new(),
        }
    }

    pub fn add_render_done_listener<T>(&mut self, listener: T)
        where T: Fn(&mut ViewModel) + 'static
    {
        let event = Event::Rendered;
        self.listeners.entry(event).or_insert(Vec::new()).push(Rc::new(listener));
    }

    pub fn render_done(&mut self) {
        self.events.push(Event::Rendered);
    }

    pub fn add_update_done_listener<T>(&mut self, listener: T)
        where T: Fn(&mut ViewModel) + 'static
    {
        let event = Event::Updated;
        self.listeners.entry(event).or_insert(Vec::new()).push(Rc::new(listener));
    }

    pub fn update_done(&mut self) {
        self.events.push(Event::Updated);
    }

    pub fn fire_events(&mut self) {
        while !self.events.is_empty() {
            let events = self.events.clone();
            self.events.clear();
            for event in events {
                self.fire_event(event)
            }
        }
    }

    fn fire_event(&mut self, event: Event) {
        let listeners = self.clone_listeners(event);
        self.notify_listeners(listeners);
    }

    fn clone_listeners(&mut self, event: Event) -> Vec<BoxedCallback> {
        self.listeners.get(&event).unwrap().clone()
    }

    pub fn notify_listeners(&mut self, listeners: Vec<BoxedCallback>) {
        for listener in listeners {
            listener(self);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_done_callback() {
        let mut view_model = ViewModel::new();
        view_model.add_render_done_listener(|view_model| { view_model.updated = true; });
        view_model.render_done();
        assert!(!view_model.updated);
        view_model.fire_events();
        assert!(view_model.updated);
    }

    #[test]
    fn update_done_callback() {
        let mut view_model = ViewModel::new();
        view_model.add_update_done_listener(|view_model| { view_model.rendered = true; });
        view_model.update_done();
        assert!(!view_model.rendered);
        view_model.fire_events();
        assert!(view_model.rendered);
    }

    #[test]
    fn chained_callbacks() {
        let mut view_model = ViewModel::new();
        view_model.add_render_done_listener(|view_model| {
            view_model.updated = true;
            view_model.update_done();
        });
        view_model.add_update_done_listener(|view_model| { view_model.rendered = true; });
        view_model.render_done();
        view_model.fire_events();
        assert!(view_model.rendered);
    }
}
