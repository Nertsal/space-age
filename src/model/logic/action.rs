use super::*;

impl Model {
    pub fn action(&mut self, action: Action) {
        if !self.abilities.contains(&Ability::Action(action.clone())) {
            return;
        }

        match action {
            Action::TheoreticResearch => {
                self.theorizing = true;
            }
            Action::Launch(ty) => self.launch_satellite(true, ty),
            Action::Deorbit => todo!(),
        }
    }

    fn launch_satellite(&mut self, pay_cost: bool, kind: SatelliteKind) {
        let Some(config) = self.config.satellites.get(&kind) else {
            log::error!("Satellite kind missing config: {:?}", kind);
            return;
        };
        if pay_cost {
            if self.science < config.launch_cost {
                return;
            }
            self.science -= config.launch_cost;
        }

        let mut rng = thread_rng();

        let orbit = &mut self.planet.orbit;
        let position = SpherePos {
            distance: orbit.distance,
            polar: random_angle(&mut rng),
            azimuth: random_angle(&mut rng),
        };
        orbit.satellites.insert(Satellite {
            kind,
            position,
            velocity: random_orbit_velocity(position, &mut rng),
            radius: r32(0.3),
            trail: VecDeque::new(),
            science_timer: Bounded::new_max(config.interval),
        });
    }

    pub fn research(&mut self, id: u64) {
        if self.researched.contains(&id) {
            return;
        }

        let Some(research) = self.config.research.items.iter().find(|item| item.id == id) else {
            return;
        };
        if self.science < research.cost {
            return;
        }

        self.science -= research.cost;
        self.researched.insert(id);
        match &research.effect {
            Research::Unlock(ability) => {
                self.abilities.insert(ability.clone());
            }
            &Research::Upgrade(stat, change) => {
                *self.stats.entry(stat).or_insert(R32::ONE) += change;
            }
        }
    }
}

pub fn random_orbit_velocity(position: SpherePos, rng: &mut impl Rng) -> SphereVelocity {
    // Find an axis perpendicular to the position to define the orbit
    let position = position.to_cartesian(vec2::ZERO);
    let perp_a = vec3(position.y, -position.x, Coord::ZERO);
    let perp_b = vec3(position.z, Coord::ZERO, -position.x);

    let a = r32(rng.gen_range(-1.0..=1.0));
    let b = r32(rng.gen_range(-1.0..=1.0));

    let axis = (perp_a * a + perp_b * b).normalize_or_zero();

    let speed = r32(rng.gen_range(0.5..0.7));
    SphereVelocity {
        speed: Angle::from_radians(speed),
        axis,
    }
}
