type BoxedCallback = Box<Fn(&mut bool)>;

pub struct ViewModel {
    pub updated: bool,
    pub rendered: bool,
    render_done_listeners: Vec<BoxedCallback>,
    update_done_listeners: Vec<BoxedCallback>,
}

impl ViewModel {
    pub fn new() -> ViewModel {
        ViewModel {
            updated: false,
            rendered: false,
            render_done_listeners: Vec::new(),
            update_done_listeners: Vec::new(),
        }
    }

    pub fn add_render_done_listener<T>(&mut self, listener: T)
        where T: Fn(&mut bool) + 'static
    {
        self.render_done_listeners.push(Box::new(listener));
    }

    pub fn render_done(&mut self) {
        for listener in &self.render_done_listeners {
            listener(&mut self.updated);
        }
    }

    pub fn add_update_done_listener<T>(&mut self, listener: T)
        where T: Fn(&mut bool) + 'static
    {
        self.update_done_listeners.push(Box::new(listener));
    }

    pub fn update_done(&mut self) {
        for listener in &self.update_done_listeners {
            listener(&mut self.rendered);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_done_callback() {
        let mut view_model = ViewModel::new();
        view_model.add_render_done_listener(|updated| { *updated = true });
        view_model.render_done();
        assert!(view_model.updated);
    }

    #[test]
    fn update_done_callback() {
        let mut view_model = ViewModel::new();
        view_model.add_update_done_listener(|rendered| { *rendered = true });
        view_model.update_done();
        assert!(view_model.rendered);
    }
}
