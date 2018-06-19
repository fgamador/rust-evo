pub struct ViewModel {}

impl ViewModel {
    pub fn new() -> ViewModel {
        ViewModel {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut view_model = ViewModel::new();
        assert_eq!(2 + 2, 4);
    }
}
