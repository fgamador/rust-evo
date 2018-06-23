use std::rc::Rc;

type BoxedCallback = Rc<Fn(&mut ViewModel) -> ()>;

#[derive(Clone)]
enum Event {
    Rendered,
    Updated,
}

pub struct ViewModel {
    pub updated: bool,
    pub rendered: bool,
    events: Vec<Event>,
    render_done_listeners: Vec<BoxedCallback>,
    update_done_listeners: Vec<BoxedCallback>,
}

impl ViewModel {
    pub fn new() -> ViewModel {
        ViewModel {
            updated: false,
            rendered: false,
            events: Vec::new(),
            render_done_listeners: Vec::new(),
            update_done_listeners: Vec::new(),
        }
    }

    pub fn add_render_done_listener<T>(&mut self, listener: T)
        where T: Fn(&mut ViewModel) + 'static
    {
        self.render_done_listeners.push(Rc::new(listener));
    }

    pub fn render_done(&mut self) {
        self.events.push(Event::Rendered);
    }

    pub fn add_update_done_listener<T>(&mut self, listener: T)
        where T: Fn(&mut ViewModel) + 'static
    {
        self.update_done_listeners.push(Rc::new(listener));
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
        match event {
            Event::Rendered => {
                let listeners = self.render_done_listeners.clone();
                self.notify_listeners(listeners);
            }
            Event::Updated => {
                let listeners = self.update_done_listeners.clone();
                self.notify_listeners(listeners);
            }
        }
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

//    #[test]
//    fn callbacks_do_not_call_each_other() {
//        let mut view_model = ViewModel::new();
//        view_model.add_render_done_listener(|updated| { *updated = true });
//        view_model.add_update_done_listener(|rendered| { *rendered = true });
//
//        view_model.render_done();
//
//        assert!(view_model.updated);
//    }
}
