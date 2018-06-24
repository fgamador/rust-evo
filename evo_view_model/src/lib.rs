use std::collections::HashMap;
use std::hash::Hash;
use std::rc::Rc;

type BoxedCallback = Rc<Fn(&mut ViewModel) -> ()>;

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub enum Event {
    Rendered,
    Updated,
}

pub struct ViewModel {
    //    event_manager: EventManager<Event, ViewModel>,
    pub updated: bool,
    pub rendered: bool,
    events: Vec<Event>,
    listeners: HashMap<Event, Vec<BoxedCallback>>,
}

impl ViewModel {
    pub fn new() -> ViewModel {
        ViewModel {
//            event_manager: EventManager::new(),
            updated: false,
            rendered: false,
            events: Vec::new(),
            listeners: HashMap::new(),
        }
    }

    pub fn add_render_done_listener<T>(&mut self, listener: T)
        where T: Fn(&mut ViewModel) + 'static
    {
//        self.event_manager.add_listener(Event::Rendered, listener);
        self.add_listener(Event::Rendered, listener);
    }

    pub fn add_update_done_listener<T>(&mut self, listener: T)
        where T: Fn(&mut ViewModel) + 'static
    {
//        self.event_manager.add_listener(Event::Updated, listener);
        self.add_listener(Event::Updated, listener);
    }

    pub fn render_done(&mut self) {
//        self.event_manager.add_event(Event::Rendered);
        self.add_event(Event::Rendered);
    }

    pub fn update_done(&mut self) {
//        self.event_manager.add_event(Event::Updated);
        self.add_event(Event::Updated);
    }

    pub fn fire_events(&mut self) {
        //self.event_manager.fire_events(self);
        while !self.events.is_empty() {
            let events = self.events.clone();
            self.events.clear();
            for event in events {
                self.fire_event(event)
            }
        }
    }
}

impl ViewModel {
    pub fn add_listener<T>(&mut self, event: Event, listener: T)
        where T: Fn(&mut ViewModel) + 'static
    {
        self.listeners.entry(event).or_insert(Vec::new()).push(Rc::new(listener));
    }

    pub fn add_event(&mut self, event: Event)
    {
        self.events.push(event);
    }

    fn fire_event(&mut self, event: Event) {
        let listeners = self.clone_listeners(event);
        ViewModel::notify_listeners(self, listeners);
    }

    fn clone_listeners(&self, event: Event) -> Vec<BoxedCallback> {
        self.listeners.get(&event).unwrap().clone()
    }

    fn notify_listeners(source: &mut ViewModel, listeners: Vec<BoxedCallback>) {
        for listener in listeners {
            listener(source);
        }
    }
}

type Callback<M, S> = Rc<Fn(&mut M, &mut S) -> ()>;

pub struct EventManager<E, S> {
    events: Vec<E>,
    listeners: HashMap<E, Vec<Callback<EventManager<E, S>, S>>>,
}

impl<E, S> EventManager<E, S> where E: Clone + Copy + Eq + Hash {
    pub fn new() -> Self {
        EventManager {
            events: Vec::new(),
            listeners: HashMap::new(),
        }
    }

    pub fn add_listener<C>(&mut self, event: E, listener: C)
        where C: Fn(&mut EventManager<E, S>, &mut S) + 'static
    {
        self.listeners.entry(event).or_insert(Vec::new()).push(Rc::new(listener));
    }

    pub fn add_event(&mut self, event: E)
    {
        self.events.push(event);
    }

    pub fn fire_events(&mut self, subject: &mut S) {
        while !self.events.is_empty() {
            let events = self.events.clone();
            self.events.clear();
            for event in events {
                self.fire_event(subject, event)
            }
        }
    }

    fn fire_event(&mut self, subject: &mut S, event: E) {
        let listeners = self.clone_listeners(event);
        self.notify_listeners(subject, listeners);
    }

    fn clone_listeners(&self, event: E) -> Vec<Callback<EventManager<E, S>, S>> {
        self.listeners.get(&event).unwrap().clone()
    }

    fn notify_listeners(&mut self, subject: &mut S, listeners: Vec<Callback<EventManager<E, S>, S>>) {
        for listener in listeners {
            listener(self, subject);
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

    #[test]
    fn event_manager() {
        let mut event_manager: EventManager<Event, ViewModel> = EventManager::new();
        let mut view_model = ViewModel::new();
        event_manager.add_listener(Event::Rendered, |event_manager, view_model| {
            view_model.updated = true;
            event_manager.add_event(Event::Updated);
        });
        event_manager.add_listener(Event::Updated, |_, view_model| {
            view_model.rendered = true;
        });
        event_manager.add_event(Event::Rendered);
        event_manager.fire_events(&mut view_model);
        assert!(view_model.rendered);
    }
}
