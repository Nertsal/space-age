use crate::model::logic::action::random_orbit_velocity;

use super::*;

impl Model {
    pub fn update(&mut self, delta_time: Time) {
        self.real_time += delta_time;
        let mut rng = thread_rng();

        self.hovered_rotation += Angle::from_degrees(r32(15.0) * delta_time);
        self.selected_rotation -= Angle::from_degrees(r32(15.0) * delta_time);

        // Theorycrafting
        // if auto_theory {
        //     self.theory_progress.change(delta_time);
        // }
        while self.theory_progress > R32::ONE {
            let stat = self.get_stat(Stat::Theorycrafting);
            self.science +=
                (self.config.theoretic_research.science as f32 * stat.as_f32()).ceil() as Science;
            self.theory_progress -= R32::ONE;
        }

        self.movement(delta_time);

        // Update satellites production
        let sat_eff = self.get_stat(Stat::SatelliteEfficiency);
        let longevity = self.get_stat(Stat::SatelliteLongevity);
        let orbit = &mut self.planet.orbit;
        for (kind, science_timer, lifetime) in
            query!(orbit.satellites, (&kind, &mut science_timer, &mut lifetime))
        {
            lifetime.change(-delta_time / longevity - r32(rng.gen_range(-0.01..=0.01)));
            if lifetime.is_min() {
                // This satellite is non-functioning
                continue;
            }

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

    fn movement(&mut self, delta_time: Time) {
        #[derive(Clone, Copy)]
        enum Id {
            Satellite(ArenaId),
            Debris(ArenaId),
        }

        let planet = &mut self.planet;
        let orbit = &mut planet.orbit;

        // Update positions
        let mut destroyed = Vec::new();
        let mut move_object = |id,
                               position: &mut SpherePos,
                               velocity: &SphereVelocity,
                               trail: &mut VecDeque<SpherePos>,
                               deorbiting: bool| {
            position.add_velocity(*velocity, delta_time);
            if deorbiting {
                position.distance -= r32(1.0) * delta_time;
                if position.distance < planet.radius {
                    destroyed.push(id);
                }
            }
            if trail.len() >= ORBIT_OBJECT_TRAIL_LEN {
                trail.pop_back();
            }
            trail.push_front(*position);
        };
        for (id, position, velocity, trail, &deorbiting) in query!(
            orbit.satellites,
            (id, &mut position, &velocity, &mut trail, &deorbiting)
        ) {
            move_object(Id::Satellite(id), position, velocity, trail, deorbiting);
        }
        for (id, position, velocity, trail, &deorbiting) in query!(
            orbit.debris,
            (id, &mut position, &velocity, &mut trail, &deorbiting)
        ) {
            move_object(Id::Debris(id), position, velocity, trail, deorbiting);
        }

        // Remove destroyed objects
        for id in destroyed {
            match id {
                Id::Satellite(id) => {
                    orbit.satellites.remove(id);
                }
                Id::Debris(id) => {
                    orbit.debris.remove(id);
                }
            }
        }

        // Check collisions
        macro_rules! get_object {
            ($arch:expr, $id:expr) => {
                get!($arch, $id, (&position, &radius))
            };
        }
        let get_object = |id: Id| match id {
            Id::Satellite(id) => get_object!(orbit.satellites, id),
            Id::Debris(id) => get_object!(orbit.debris, id),
        };

        // Find collisions
        let mut collisions = Vec::new();
        let mut check = |id_a, id_b| {
            if let Some((pos_a, &rad_a)) = get_object(Id::Satellite(id_a))
                && let Some((pos_b, &rad_b)) = get_object(id_b)
            {
                let delta = pos_b.to_cartesian(vec2::ZERO) - pos_a.to_cartesian(vec2::ZERO);
                let distance = delta.len();
                if distance < rad_a + rad_b {
                    collisions.push((id_a, id_b));
                }
            }
        };
        let satellite_ids: Vec<_> = orbit.satellites.ids().collect();
        for (&id_a, &id_b) in itertools::izip![&satellite_ids, satellite_ids.iter().skip(1)] {
            check(id_a, Id::Satellite(id_b));
            for id_b in orbit.debris.ids() {
                check(id_a, Id::Debris(id_b));
            }
        }

        // Resolve collisions
        let mut rng = thread_rng();
        for id in collisions
            .into_iter()
            .flat_map(|(a, b)| {
                let b = if let Id::Satellite(b) = b {
                    Some(b)
                } else {
                    None
                };
                [Some(a), b]
            })
            .flatten()
        {
            if let Some(satellite) = orbit.satellites.remove(id) {
                let mut trail = Some(satellite.trail);
                for _ in 0..4 {
                    orbit.debris.insert(Debris {
                        position: satellite.position,
                        velocity: random_orbit_velocity(satellite.position, &mut rng),
                        visual_radius: satellite.visual_radius / r32(2.0),
                        radius: satellite.radius / r32(4.0),
                        trail: trail.take().unwrap_or_default(),
                        deorbiting: false,
                    });
                }
            }
        }
    }
}
