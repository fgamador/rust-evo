use crate::physics::quantities::*;

pub trait NewtonianBody {
    fn mass(&self) -> Mass;
    fn position(&self) -> Position;
    fn velocity(&self) -> Velocity;
    fn move_for_one_tick(&mut self);
    fn kick(&mut self, impulse: Impulse);
    fn net_force(&self) -> &NetForce;
    fn net_force_mut(&mut self) -> &mut NetForce;
    fn exert_net_force_for_one_tick(&mut self);
}

#[derive(Clone, Debug)]
pub struct NewtonianState {
    pub mass: Mass,
    pub position: Position,
    pub velocity: Velocity,
    pub net_force: NetForce,
}

impl NewtonianState {
    pub fn new(mass: Mass, position: Position, velocity: Velocity) -> NewtonianState {
        NewtonianState {
            mass,
            position,
            velocity,
            net_force: NetForce::ZERO,
        }
    }
}

impl NewtonianBody for NewtonianState {
    fn mass(&self) -> Mass {
        self.mass
    }

    fn position(&self) -> Position {
        self.position
    }

    fn velocity(&self) -> Velocity {
        self.velocity
    }

    fn move_for_one_tick(&mut self) {
        self.position = self.position + self.velocity * Duration::ONE;
    }

    fn kick(&mut self, impulse: Impulse) {
        self.velocity = self.velocity + impulse / self.mass;
    }

    fn net_force(&self) -> &NetForce {
        &self.net_force
    }

    fn net_force_mut(&mut self) -> &mut NetForce {
        &mut self.net_force
    }

    fn exert_net_force_for_one_tick(&mut self) {
        let impulse = self.net_force.net_force() * Duration::ONE;
        self.kick(impulse);
    }
}

#[derive(Clone, Debug)]
pub struct NetForce {
    dominant_force: Force,
    dominant_force_label: &'static str,
    non_dominant_forces: Force,
    non_dominant_force_additions: Option<Vec<ForceAddition>>,
}

impl NetForce {
    pub const ZERO: NetForce = NetForce {
        dominant_force: Force::ZERO,
        dominant_force_label: "",
        non_dominant_forces: Force::ZERO,
        non_dominant_force_additions: None,
    };

    pub fn start_recording_force_additions(&mut self) {
        self.non_dominant_force_additions = Some(vec![]);
    }

    pub fn stop_recording_force_additions(&mut self) {
        self.non_dominant_force_additions = None;
    }

    pub fn add_dominant_force(&mut self, force: Force, _label: &'static str) {
        // if force.value().dot_sqr() > self.dominant_force.value().dot_sqr() {
        //     self.dominant_force = force;
        //     self.dominant_force_label = label;
        // }

        self.dominant_force = Force::new(
            Self::stronger(force.x(), self.dominant_force.x()),
            Self::stronger(force.y(), self.dominant_force.y()),
        );
    }

    fn stronger(lhs: f64, rhs: f64) -> f64 {
        if lhs.abs() > rhs.abs() {
            lhs
        } else {
            rhs
        }
    }

    pub fn add_non_dominant_force(&mut self, force: Force, label: &'static str) {
        self.non_dominant_forces += force;

        if let Some(force_additions) = &mut self.non_dominant_force_additions {
            force_additions.push(ForceAddition { force, label });
        }
    }

    pub fn clear(&mut self) {
        self.dominant_force = Force::ZERO;
        self.non_dominant_forces = Force::ZERO;

        if let Some(force_additions) = &mut self.non_dominant_force_additions {
            force_additions.clear();
        }
    }

    pub fn net_force(&self) -> Force {
        self.dominant_force + self.non_dominant_forces
    }

    pub fn dominant_force(&self) -> Force {
        self.dominant_force
    }

    pub fn dominant_force_label(&self) -> &'static str {
        self.dominant_force_label
    }

    pub fn non_dominant_force_additions(&self) -> &Option<Vec<ForceAddition>> {
        &self.non_dominant_force_additions
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ForceAddition {
    pub force: Force,
    pub label: &'static str,
}

#[cfg(test)]
mod tests {
    use super::*;
    use evo_domain_derive::NewtonianBody;

    #[test]
    fn coasting() {
        let mut subject = SimpleBody::new(
            Mass::new(2.0),
            Position::new(-1.0, 1.5),
            Velocity::new(1.0, 2.0),
        );
        subject.move_for_one_tick();
        assert_eq!(subject.position(), Position::new(0.0, 3.5));
        assert_eq!(subject.velocity(), Velocity::new(1.0, 2.0));
    }

    #[test]
    fn kicked() {
        let mut subject = SimpleBody::new(
            Mass::new(2.0),
            Position::new(-1.0, 2.0),
            Velocity::new(1.0, -1.0),
        );
        subject.kick(Impulse::new(0.5, 0.5));
        assert_eq!(subject.position(), Position::new(-1.0, 2.0));
        assert_eq!(subject.velocity(), Velocity::new(1.25, -0.75));
    }

    #[test]
    fn non_dominant_forces_add() {
        let mut subject = NetForce::ZERO;
        subject.add_non_dominant_force(Force::new(1.5, -0.5), "test");
        subject.add_non_dominant_force(Force::new(0.25, -0.5), "test");
        assert_eq!(subject.net_force(), Force::new(1.75, -1.0));
    }

    #[test]
    fn non_dominant_forces_add_to_dominant_force() {
        let mut subject = NetForce::ZERO;
        subject.add_non_dominant_force(Force::new(1.5, -0.5), "test");
        subject.add_dominant_force(Force::new(3.5, -1.5), "test");
        assert_eq!(subject.net_force(), Force::new(5.0, -2.0));
    }

    #[test]
    #[ignore]
    fn longer_dominant_force_is_retained() {
        let mut subject = NetForce::ZERO;
        subject.add_dominant_force(Force::new(4.0, -4.0), "test");
        subject.add_dominant_force(Force::new(4.5, 0.0), "test");
        assert_eq!(subject.net_force(), Force::new(4.0, -4.0));
    }

    #[test]
    fn clears_dominant_and_non_dominant_forces() {
        let mut subject = NetForce::ZERO;
        subject.add_dominant_force(Force::new(3.5, -1.5), "test");
        subject.add_non_dominant_force(Force::new(1.5, -0.5), "test");
        subject.clear();
        assert_eq!(subject.net_force(), Force::ZERO);
    }

    #[test]
    fn exert_net_force_for_one_tick() {
        let mut ball = SimpleBody::new(
            Mass::new(1.0),
            Position::new(1.0, 1.0),
            Velocity::new(1.0, 1.0),
        );
        ball.state
            .net_force
            .add_non_dominant_force(Force::new(1.0, 1.0), "test");
        ball.exert_net_force_for_one_tick();
        assert_eq!(ball.velocity(), Velocity::new(2.0, 2.0));
    }

    #[derive(NewtonianBody)]
    struct SimpleBody {
        state: NewtonianState,
    }

    impl SimpleBody {
        fn new(mass: Mass, position: Position, velocity: Velocity) -> SimpleBody {
            SimpleBody {
                state: NewtonianState::new(mass, position, velocity),
            }
        }
    }
}
