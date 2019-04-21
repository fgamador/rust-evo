use crate::physics::overlap::*;

pub trait HasLocalEnvironment {
    fn environment(&self) -> &LocalEnvironment;

    fn environment_mut(&mut self) -> &mut LocalEnvironment;
}

#[derive(Clone, Debug)]
pub struct LocalEnvironment {
    overlaps: Vec<Overlap>,
    light_intensity: f64,
}

impl LocalEnvironment {
    pub fn new() -> Self {
        LocalEnvironment {
            overlaps: vec![],
            light_intensity: 0.0,
        }
    }

    pub fn add_overlap(&mut self, overlap: Overlap) {
        self.overlaps.push(overlap);
    }

    pub fn overlaps(&self) -> &Vec<Overlap> {
        &self.overlaps
    }

    pub fn add_light_intensity(&mut self, light_intensity: f64) {
        self.light_intensity += light_intensity;
    }

    pub fn light_intensity(&self) -> f64 { self.light_intensity }

    pub fn clear(&mut self) {
        self.overlaps.clear();
        self.light_intensity = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::physics::quantities::*;

    #[test]
    fn add_overlap() {
        let mut env = LocalEnvironment::new();
        env.add_overlap(Overlap::new(Displacement::new(1.0, 1.0)));
        env.add_overlap(Overlap::new(Displacement::new(1.0, 1.0)));
        assert_eq!(2, env.overlaps().len());
    }

    #[test]
    fn add_light_intensity() {
        let mut env = LocalEnvironment::new();
        env.add_light_intensity(1.0);
        env.add_light_intensity(1.0);
        assert_eq!(2.0, env.light_intensity());
    }

    #[test]
    fn clear_local_environment() {
        let mut env = LocalEnvironment::new();
        env.add_overlap(Overlap::new(Displacement::new(1.0, 1.0)));
        env.add_light_intensity(1.0);

        env.clear();

        assert!(env.overlaps().is_empty());
        assert_eq!(0.0, env.light_intensity());
    }
}
