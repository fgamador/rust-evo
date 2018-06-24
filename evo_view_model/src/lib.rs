use std::collections::HashMap;
use std::hash::Hash;
use std::rc::Rc;

type Callback<Q, S> = Fn(&mut Q, &mut S) -> ();
type CallbackVec<E, S> = Vec<Rc<Callback<EventQueue<E>, S>>>;

pub struct EventManager<E, S> {
    events: EventQueue<E>,
    listeners: HashMap<E, CallbackVec<E, S>>,
}

pub struct EventQueue<E> {
    events: Vec<E>,
}

impl<E> EventQueue<E> where E: Clone + Copy {
    fn new() -> Self {
        EventQueue {
            events: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    pub fn push(&mut self, event: E) {
        self.events.push(event);
    }

    fn clone_and_clear_events(&mut self) -> Vec<E> {
        let cloned = self.events.clone();
        self.events.clear();
        cloned
    }
}

impl<E, S> EventManager<E, S> where E: Clone + Copy + Eq + Hash {
    pub fn new() -> Self {
        EventManager {
            events: EventQueue::new(),
            listeners: HashMap::new(),
        }
    }

    pub fn add_listener<C>(&mut self, event: E, listener: C)
        where C: Fn(&mut EventQueue<E>, &mut S) + 'static
    {
        self.listeners.entry(event).or_insert(Vec::new()).push(Rc::new(listener));
    }

    pub fn events(&mut self) -> &mut EventQueue<E> {
        &mut self.events
    }

    pub fn fire_events(&mut self, subject: &mut S) {
        while !self.events.is_empty() {
            for event in self.events.clone_and_clear_events() {
                self.fire_event(subject, event)
            }
        }
    }

    fn fire_event(&mut self, subject: &mut S, event: E) {
        for listener in self.listeners.get(&event).unwrap() {
            listener(&mut self.events, subject);
        }
    }
}

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
        event_manager.events().push(Event::Rendered);
        assert!(!view_model.updated);
        event_manager.fire_events(&mut view_model);
        assert!(view_model.updated);
    }

    #[test]
    fn chained_callbacks() {
        let mut event_manager: EventManager<Event, ViewModel> = EventManager::new();
        let mut view_model = ViewModel::new();
        event_manager.add_listener(Event::Rendered, |event_queue, _| {
            event_queue.push(Event::Updated);
        });
        event_manager.add_listener(Event::Updated, |_, view_model| {
            view_model.rendered = true;
        });
        event_manager.events().push(Event::Rendered);
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
//        event_manager.events().push(Event::Rendered);
//        assert!(!callback_ran);
//        event_manager.fire_events(&mut view_model);
//        assert!(callback_ran);
//    }
}
