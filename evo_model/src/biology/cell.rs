use biology::layers::*;
use environment::environment::*;
use evo_view_model::Color;
use physics::newtonian::*;
use physics::quantities::*;
use physics::shapes::*;
use physics::sortable_graph::*;
use std::f64::consts::PI;
use std::ptr;

#[derive(Debug, GraphNode, HasLocalEnvironment, NewtonianBody)]
pub struct Cell {
    graph_node_data: GraphNodeData,
    radius: Length,
    newtonian_state: NewtonianState,
    environment: LocalEnvironment,
    layers: Vec<Box<CellLayer>>,
}

impl Cell {
    pub fn new(position: Position, velocity: Velocity, mut layers: Vec<Box<CellLayer>>) -> Self {
        if layers.is_empty() {
            panic!("Cell must have at least one layer");
        }

        let radius = Self::update_layer_outer_radii(&mut layers);
        Cell {
            graph_node_data: GraphNodeData::new(),
            radius,
            newtonian_state: NewtonianState::new(Self::calc_mass(&layers), position, velocity),
            environment: LocalEnvironment::new(),
            layers: vec!(Box::new(SimpleCellLayer::new(
                PI * radius.sqr(), Density::new(1.0), Color::Green))),
        }
    }

    fn update_layer_outer_radii(layers: &mut Vec<Box<CellLayer>>) -> Length {
        layers.iter_mut().fold(
            Length::new(0.0),
            |inner_radius, layer| {
                layer.update_outer_radius(inner_radius);
                layer.outer_radius()
            })
    }

    fn calc_mass(layers: &Vec<Box<CellLayer>>) -> Mass {
        layers.iter().fold(
            Mass::new(0.0), |mass, layer| mass + layer.mass())
    }
}

impl PartialEq for Cell {
    fn eq(&self, other: &Self) -> bool {
        ptr::eq(self, other)
    }
}

impl Circle for Cell {
    fn radius(&self) -> Length {
        self.radius
    }

    fn center(&self) -> Position {
        self.newtonian_state.position
    }
}

impl Onion for Cell {
    type Layer = CellLayer;

    fn layers(&self) -> &[Box<Self::Layer>] {
        &self.layers
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cells_use_pointer_equality() {
        let cell1 = Cell::new(Position::new(1.0, 1.0), Velocity::new(1.0, 1.0),
                              vec![
                                  Box::new(SimpleCellLayer::new(
                                      Area::new(PI), Density::new(1.0), Color::Green))
                              ]);
        let cell2 = Cell::new(Position::new(1.0, 1.0), Velocity::new(1.0, 1.0),
                              vec![
                                  Box::new(SimpleCellLayer::new(
                                      Area::new(PI), Density::new(1.0), Color::Green))
                              ]);
        assert_eq!(cell1, cell1);
        assert_ne!(cell1, cell2);
    }

    #[test]
    #[should_panic]
    fn cell_must_have_layers() {
        Cell::new(Position::new(1.0, 1.0), Velocity::new(1.0, 1.0), vec![]);
    }

    #[test]
    fn cell_has_radius_of_outer_layer() {
        let cell = Cell::new(Position::new(1.0, 1.0), Velocity::new(1.0, 1.0),
                             vec![
                                 Box::new(SimpleCellLayer::new(
                                     Area::new(PI), Density::new(1.0), Color::Green)),
                                 Box::new(SimpleCellLayer::new(
                                     Area::new(3.0 * PI), Density::new(1.0), Color::Green))
                             ]);
        assert_eq!(Length::new(2.0), cell.radius());
    }

    #[test]
    fn cell_has_mass_of_all_layers() {
        let cell = Cell::new(Position::new(1.0, 1.0), Velocity::new(1.0, 1.0),
                             vec![
                                 Box::new(SimpleCellLayer::new(
                                     Area::new(PI), Density::new(1.0), Color::Green)),
                                 Box::new(SimpleCellLayer::new(
                                     Area::new(2.0 * PI), Density::new(2.0), Color::Green))
                             ]);
        assert_eq!(Mass::new(5.0 * PI), cell.mass());
    }
}
