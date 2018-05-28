use std::f64::consts::PI;

pub trait Circular {
    fn area(&self) -> f64;
}

pub struct Circle {
    pub radius: f64,
}

impl Circular for Circle {
    fn area(&self) -> f64 {
        PI * self.radius * self.radius
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    #[test]
    fn circle_knows_area() {
        let circle = Circle { radius: 2.0 };
        assert_eq!(PI * 4.0, circle.area());
    }
}
