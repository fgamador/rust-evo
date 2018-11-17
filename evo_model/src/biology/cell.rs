use biology::layers::*;
use environment::environment::*;
use physics::newtonian::*;
use physics::quantities::*;
use physics::shapes::*;
use physics::sortable_graph::*;
use std::ptr;

#[derive(Clone, Debug, GraphNode, HasLocalEnvironment, NewtonianBody)]
pub struct Cell {
    graph_node_data: GraphNodeData,
    radius: Length,
    newtonian_state: NewtonianState,
    environment: LocalEnvironment,
}

impl Cell {
    pub fn new(position: Position, velocity: Velocity, mut layers: Vec<SimpleCellLayer>) -> Self {
        if layers.is_empty() {
            panic!("Cell must have layers");
        }
        for (i, layer) in layers.iter().enumerate() {
            if i > 0 && layer.outer_radius() < layers[i - 1].outer_radius() {
                panic!("Cell layers must be non-decreasing");
            }
        }

        layers.iter_mut().fold(
            Length::new(0.0),
            |inner_radius, layer| layer.update_outer_radius(inner_radius));
        let mass = layers.iter().fold(
            Mass::new(0.0), |mass, layer| mass + layer.mass());
        Cell {
            graph_node_data: GraphNodeData::new(),
            radius: layers.last().unwrap().outer_radius(),
            newtonian_state: NewtonianState::new(mass, position, velocity),
            environment: LocalEnvironment::new(),
        }
    }

    pub fn new_old(radius: Length, mass: Mass, position: Position, velocity: Velocity) -> Cell {
        Cell {
            graph_node_data: GraphNodeData::new(),
            radius,
            newtonian_state: NewtonianState::new(mass, position, velocity),
            environment: LocalEnvironment::new(),
        }
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
    // TODO rings
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    #[test]
    fn cells_use_pointer_equality() {
        let cell1 = Cell::new_old(Length::new(1.0), Mass::new(1.0),
                                  Position::new(1.0, 1.0), Velocity::new(1.0, 1.0));
        let cell2 = Cell::new_old(Length::new(1.0), Mass::new(1.0),
                                  Position::new(1.0, 1.0), Velocity::new(1.0, 1.0));
        assert_eq!(cell1, cell1);
        assert_ne!(cell1, cell2);
    }

    #[test]
    #[should_panic]
    fn cell_must_have_layers() {
        Cell::new(Position::new(1.0, 1.0), Velocity::new(1.0, 1.0), vec![]);
    }

    #[test]
    #[should_panic]
    fn cell_layers_must_be_non_decreasing() {
        Cell::new(Position::new(1.0, 1.0), Velocity::new(1.0, 1.0),
                  vec![
                      SimpleCellLayer::new_old(Length::new(2.0), Density::new(1.0)),
                      SimpleCellLayer::new_old(Length::new(1.0), Density::new(1.0))
                  ]);
    }

    #[test]
    fn cell_has_radius_of_outer_layer() {
        let cell = Cell::new(Position::new(1.0, 1.0), Velocity::new(1.0, 1.0),
                             vec![
                                 SimpleCellLayer::new(Area::new(PI), Density::new(1.0)),
                                 SimpleCellLayer::new(Area::new(3.0 * PI), Density::new(1.0))
                             ]);
        assert_eq!(Length::new(2.0), cell.radius());
    }

    #[test]
    fn cell_has_mass_of_all_layers() {
        let cell = Cell::new(Position::new(1.0, 1.0), Velocity::new(1.0, 1.0),
                             vec![
                                 SimpleCellLayer::new(Area::new(PI), Density::new(1.0)),
                                 SimpleCellLayer::new(Area::new(2.0 * PI), Density::new(2.0))
                             ]);
        assert_eq!(Mass::new(5.0 * PI), cell.mass());
    }
}
