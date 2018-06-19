pub struct ViewModel {}

type BoxedCallback = Box<Fn(&ViewModel)>;

impl ViewModel {
    pub fn new() -> ViewModel {
        ViewModel {}
    }

    pub fn add_render_done_listener<C>(&mut self, listener: C)
        where C: Fn(&ViewModel)
    {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut view_model = ViewModel::new();
        view_model.add_render_done_listener(|view_model| {});
        assert_eq!(2 + 2, 4);
    }
}
