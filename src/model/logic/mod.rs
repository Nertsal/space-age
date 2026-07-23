use super::*;

use crate::util::random_angle;

impl Model {
    pub fn init(&mut self) {}

    pub fn update(&mut self, delta_time: Time) {
        self.real_time += delta_time;
        let mut rng = thread_rng();

        // Update positions
        let orbit = &mut self.planet.orbit;
        for (position, velocity, trail) in query!(
            [orbit.satellites, orbit.debris],
            (&mut position, &velocity, &mut trail)
        ) {
            position.add_velocity(*velocity, delta_time);
            if trail.len() >= ORBIT_OBJECT_TRAIL_LEN {
                trail.pop_back();
            }
            trail.push_front(*position);
        }

        // Update satellites production
        let sat_eff = self.get_stat(Stat::SatelliteEfficiency);
        let orbit = &mut self.planet.orbit;
        for (kind, science_timer) in query!(orbit.satellites, (&kind, &mut science_timer)) {
            let config = self
                .config
                .satellites
                .get(kind)
                .cloned()
                .unwrap_or_default();
            // NOTE: rng timer to desynchronise satelites so each one gives science at a different time
            science_timer.change(-delta_time - r32(rng.gen_range(-0.01..=0.01)));
            if science_timer.is_min() {
                science_timer.set_ratio(Time::ONE);
                self.science += (config.science as f32 * sat_eff.as_f32()).ceil() as Science;
            }
        }
    }

    pub fn action(&mut self, action: Action) {
        if !self.abilities.contains(&Ability::Action(action.clone())) {
            return;
        }

        match action {
            Action::TheoreticResearch => {
                let stat = self.get_stat(Stat::Theorycrafting);
                self.science +=
                    (self.config.theoretic_research as f32 * stat.as_f32()).ceil() as Science;
            }
            Action::Launch(ty) => self.launch_satellite(true, ty),
            Action::Deorbit => todo!(),
        }
    }

    pub fn get_stat(&self, stat: Stat) -> R32 {
        self.stats.get(&stat).copied().unwrap_or(R32::ONE)
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
            velocity: {
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
            },
            radius: r32(0.3),
            trail: VecDeque::new(),
            science_timer: Bounded::new_max(config.interval),
        });
    }

    pub fn get_research_state(&self, id: u64) -> ResearchState {
        if self.researched.contains(&id) {
            return ResearchState::Researched;
        }

        let Some(research) = self.config.research.items.iter().find(|item| item.id == id) else {
            return ResearchState::Locked;
        };
        if research.after.iter().all(|id| self.researched.contains(id)) {
            ResearchState::Available {
                cost: research.cost,
            }
        } else {
            ResearchState::Locked
        }
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
