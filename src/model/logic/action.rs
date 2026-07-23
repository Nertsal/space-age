use super::*;

impl Model {
    pub fn action(&mut self, action: Action) {
        if !self.abilities.contains(&action.ability()) {
            return;
        }

        match action {
            Action::TheoreticResearch => {
                self.theory_progress += self.config.theoretic_research.clicks.recip();
            }
            Action::Launch(ty) => self.launch_satellite(true, ty),
            Action::Deorbit(target) => self.deorbit(target),
        }
    }

    fn deorbit(&mut self, target: InteractiveId) {
        let planet = &mut self.planet;
        let orbit = &mut planet.orbit;
        let deorbiting = match target {
            InteractiveId::Satellite(id) => {
                get!(orbit.satellites, id, (&mut deorbiting))
            }
            InteractiveId::Debris(id) => {
                get!(orbit.debris, id, (&mut deorbiting))
            }
        };
        if let Some(deorbiting) = deorbiting {
            *deorbiting = true;
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
            visual_radius: r32(0.3),
            radius: r32(0.15),
            trail: VecDeque::new(),
            science_timer: Bounded::new_max(config.interval),
            lifetime: Bounded::new_max(config.lifetime),
            deorbiting: false,
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

    pub fn update_cursor(&mut self, cursor_pos: vec2<Coord>) {
        let old_hover = self.hovered_object.take();

        // Find the hovered object
        let mut closest_dist = Coord::ZERO;
        let leeway = r32(0.5);

        let planet = &self.planet;
        let planet_pos = planet.position.to_cartesian();

        let mut check_hover = |id, pos: &SpherePos, radius| -> bool {
            let pos = pos.to_cartesian(vec2::ZERO);
            let behind_planet = pos.z < Coord::ZERO && pos.xy().len() < planet.radius;
            let distance = (pos.xy() + planet_pos - cursor_pos).len();
            if !behind_planet && distance < radius + leeway {
                if self.hovered_object.is_none() || distance < closest_dist {
                    closest_dist = distance;
                    self.hovered_object = Some(id);
                    true
                } else {
                    false
                }
            } else {
                false
            }
        };

        let orbit = &planet.orbit;
        for (id, pos, &radius) in query!(orbit.satellites, (id, &position, &visual_radius)) {
            if check_hover(InteractiveId::Satellite(id), pos, radius) {
                return;
            }
        }
        for (id, pos, &radius) in query!(orbit.debris, (id, &position, &visual_radius)) {
            if check_hover(InteractiveId::Debris(id), pos, radius) {
                return;
            }
        }

        // Update hover
        if old_hover != self.hovered_object {
            self.hovered_rotation = random_angle(&mut thread_rng());
        }
    }

    pub fn select_hovered(&mut self) {
        self.selected_object = self.hovered_object;
        self.selected_rotation = random_angle(&mut thread_rng());
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
