use std::collections::HashMap;
use std::hash::Hash;

type Callback<Q, S> = Fn(&mut Q, &mut S) -> ();
type CallbackVec<E, S> = Vec<Box<Callback<EventQueue<E>, S>>>;

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
        self.listeners.entry(event).or_insert(Vec::new()).push(Box::new(listener));
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
        if let Some(listeners) = self.listeners.get(&event) {
            for listener in listeners {
                listener(&mut self.events, subject);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_with_no_callback() {
        let mut event_manager: EventManager<Event, EventSubject> = EventManager::new();
        let mut event_subject = EventSubject { updated: false };
        event_manager.events().push(Event::Event1);
        event_manager.fire_events(&mut event_subject);
        assert!(!event_subject.updated);
    }

    #[test]
    fn event_with_single_callback() {
        let mut event_manager: EventManager<Event, EventSubject> = EventManager::new();
        let mut event_subject = EventSubject { updated: false };
        event_manager.add_listener(Event::Event1, |_, subject| {
            subject.updated = true;
        });
        event_manager.events().push(Event::Event1);
        assert!(!event_subject.updated);
        event_manager.fire_events(&mut event_subject);
        assert!(event_subject.updated);
    }

    #[test]
    fn event_with_chained_callbacks() {
        let mut event_manager: EventManager<Event, EventSubject> = EventManager::new();
        let mut event_subject = EventSubject { updated: false };
        event_manager.add_listener(Event::Event1, |event_queue, _| {
            event_queue.push(Event::Event2);
        });
        event_manager.add_listener(Event::Event2, |_, subject| {
            subject.updated = true;
        });
        event_manager.events().push(Event::Event1);
        event_manager.fire_events(&mut event_subject);
        assert!(event_subject.updated);
    }

    #[derive(Clone, Copy, Eq, Hash, PartialEq)]
    pub enum Event {
        Event1,
        Event2,
    }

    pub struct EventSubject {
        pub updated: bool,
    }
}
