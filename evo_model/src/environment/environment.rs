use physics::overlap::*;

pub trait HasLocalEnvironment {
    fn environment(&self) -> &LocalEnvironment;

    fn environment_mut(&mut self) -> &mut LocalEnvironment;
}

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

#[cfg(test)]
mod tests {
    //use super::*;
}
