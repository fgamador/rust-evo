use physics::quantities::*;
use std::fmt::Debug;

pub trait CellControl: Debug {}

#[derive(Debug)]
pub struct NullControl {}

impl NullControl {
    pub fn new() -> Self {
        NullControl {}
    }
}

impl CellControl for NullControl {}

#[derive(Debug)]
pub struct CyclicResizeControl {
    layer_index: usize,
    growth_ticks: u32,
    growth_fraction: f64,
}

impl CyclicResizeControl {
    pub fn new(layer_index: usize, growth_ticks: u32, growth_fraction: f64) -> Self {
        CyclicResizeControl {
            layer_index,
            growth_ticks,
            growth_fraction,
        }
    }
}

impl CellControl for CyclicResizeControl {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layer_calculates_mass() {}
}
