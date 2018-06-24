use std::collections::HashMap;
use std::hash::Hash;
use std::rc::Rc;

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub enum Event {
    Rendered,
    Updated,
}

pub struct ViewModel {
    pub updated: bool,
    pub rendered: bool,
}

impl ViewModel {
    pub fn new() -> ViewModel {
        ViewModel {
            updated: false,
            rendered: false,
        }
    }
}

type Callback<M, S> = Fn(&mut M, &mut S) -> ();
type CallbackVec<E, S> = Vec<Rc<Callback<EventManager<E, S>, S>>>;

pub struct EventManager<E, S> {
    events: Vec<E>,
    listeners: HashMap<E, CallbackVec<E, S>>,
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

    fn clone_listeners(&self, event: E) -> CallbackVec<E, S> {
        self.listeners.get(&event).unwrap().clone()
    }

    fn notify_listeners(&mut self, subject: &mut S, listeners: CallbackVec<E, S>) {
        for listener in listeners {
            listener(self, subject);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_callback() {
        let mut event_manager: EventManager<Event, ViewModel> = EventManager::new();
        let mut view_model = ViewModel::new();
        event_manager.add_listener(Event::Rendered, |_, view_model| {
            view_model.updated = true;
        });
        event_manager.add_event(Event::Rendered);
        assert!(!view_model.updated);
        event_manager.fire_events(&mut view_model);
        assert!(view_model.updated);
    }

    #[test]
    fn chained_callbacks() {
        let mut event_manager: EventManager<Event, ViewModel> = EventManager::new();
        let mut view_model = ViewModel::new();
        event_manager.add_listener(Event::Rendered, |event_manager, _| {
            event_manager.add_event(Event::Updated);
        });
        event_manager.add_listener(Event::Updated, |_, view_model| {
            view_model.rendered = true;
        });
        event_manager.add_event(Event::Rendered);
        event_manager.fire_events(&mut view_model);
        assert!(view_model.rendered);
    }

//    #[test]
//    fn callback_with_closure() {
//        let mut event_manager: EventManager<Event, ViewModel> = EventManager::new();
//        let mut view_model = ViewModel::new();
//        let mut callback_ran = false;
//        event_manager.add_listener(Event::Rendered, |_, _| {
//            callback_ran = true;
//        });
//        event_manager.add_event(Event::Rendered);
//        assert!(!callback_ran);
//        event_manager.fire_events(&mut view_model);
//        assert!(callback_ran);
//    }
}
