pub struct ViewModel {}

//type BoxedCallback = Box<FnMut(&ViewModel)>;

impl ViewModel {
    pub fn new() -> ViewModel {
        ViewModel {}
    }

    pub fn add_render_done_listener<C>(&mut self, _listener: C)
        where C: FnMut(&ViewModel)
    {}

    pub fn render_done(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_done_callback() {
        let mut view_model = ViewModel::new();
        let mut listener_notified = false;
        view_model.add_render_done_listener(|_view_model| { listener_notified = true });
        view_model.render_done();
//        assert!(listener_notified);
    }
}
