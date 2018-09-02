use physics::overlap::*;
use physics::quantities::*;

#[derive(Clone, Debug)]
pub struct LocalEnvironment {
    overlaps: Vec<Overlap>,
}

impl LocalEnvironment {
    pub fn new() -> Self {
        LocalEnvironment { overlaps: vec![] }
    }

    pub fn add_overlap(&mut self, overlap: Overlap) {
        self.overlaps.push(overlap);
    }

    pub fn overlaps(&self) -> &Vec<Overlap> {
        &self.overlaps
    }

    pub fn clear(&mut self) {
        self.overlaps.clear();
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Forces {
    net_force: Force,
}

impl Forces {
    pub fn new(initial_x: f64, initial_y: f64) -> Forces {
        Forces { net_force: Force::new(initial_x, initial_y) }
    }

    pub fn add_force(&mut self, f: Force) {
        self.net_force += f;
    }

    pub fn clear(&mut self) {
        self.net_force = Force::new(0.0, 0.0);
    }

    pub fn net_force(&self) -> Force {
        self.net_force
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn net_force() {
        let mut subject = Forces::new(1.5, -0.5);
        subject.add_force(Force::new(0.25, -0.5));
        assert_eq!(Force::new(1.75, -1.0), subject.net_force());
    }

    #[test]
    fn clear_net_force() {
        let mut subject = Forces::new(1.5, -0.5);
        subject.clear();
        assert_eq!(Force::new(0.0, 0.0), subject.net_force());
    }
}
