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
            let gained =
                (self.config.theoretic_research.science as f32 * stat.as_f32()).ceil() as Science;
            self.science += gained;
            self.theory_progress -= R32::ONE;
            self.texticles.insert(FloatingText {
                text: format!("+{}", gained).into(),
                position: (self.science_counter_pos + vec2(0.0, 1.0).as_r32()).extend(Coord::ZERO),
                velocity: vec3(1.0, 0.0, 0.0).as_r32(),
                size: r32(2.0),
                color: Color::try_from("#2AFC98").unwrap(),
                lifetime: Bounded::new_max(r32(1.0)),
            });
        }

        self.movement(delta_time);

        // Update satellites production
        let sat_eff = self.get_stat(Stat::SatelliteEfficiency);
        let longevity = self.get_stat(Stat::SatelliteLongevity);
        let orbit = &mut self.planet.orbit;
        for (kind, science_timer, lifetime) in
            query!(orbit.satellites, (&kind, &mut science_timer, &mut lifetime))
        {
            if lifetime.is_min() {
                // This satellite is non-functioning
                continue;
            }
            lifetime.change(-delta_time / longevity - r32(rng.gen_range(-0.01..=0.01)));

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
                let gained = (config.science as f32 * sat_eff.as_f32()).ceil() as Science;
                self.science += gained;
                self.texticles.insert(FloatingText {
                    text: format!("+{}", gained).into(),
                    position: (self.science_counter_pos + vec2(0.0, 1.0).as_r32())
                        .extend(Coord::ZERO),
                    velocity: vec3(1.0, 0.0, 0.0).as_r32(),
                    size: r32(2.0),
                    color: Color::try_from("#2AFC98").unwrap(),
                    lifetime: Bounded::new_max(r32(1.0)),
                });
            }
        }

        self.update_particles(delta_time);
    }

    fn movement(&mut self, delta_time: Time) {
        #[derive(Clone, Copy)]
        enum Id {
            Satellite(ArenaId),
            Debris(ArenaId),
        }

        let collision_risk = self.collision_risk();
        let planet = &mut self.planet;
        let planet_pos = planet.position.to_cartesian();
        let orbit = &mut planet.orbit;

        // Update positions
        let mut destroyed = Vec::new();
        let mut move_object = |id,
                               position: &mut SpherePos,
                               velocity: &SphereVelocity,
                               radius: &mut Coord,
                               trail: &mut VecDeque<SpherePos>,
                               deorbiting: bool| {
            position.add_velocity(*velocity, delta_time);
            if deorbiting {
                position.distance -= r32(1.0) * delta_time;
                if position.distance < planet.radius {
                    destroyed.push(id);
                }
                let pos = position.to_cartesian(planet_pos);
                let options = SpawnParticles {
                    density: r32(1.0),
                    distribution: ParticleDistribution::Circle {
                        center: pos.xy(),
                        radius: r32(0.3),
                    },
                    z: pos.z,
                    color: Color::try_from("#ADB3C2aa").unwrap(),
                    ..default()
                };
                if position.distance < planet.radius + r32(1.5) {
                    // Burning particles (realistic)
                    *radius -= *radius * r32(0.2) * delta_time;
                    self.queued_particles.extend([
                        SpawnParticles {
                            color: Color::try_from("#F45866aa").unwrap(),
                            ..options.clone()
                        },
                        SpawnParticles {
                            color: Color::try_from("#F57932aa").unwrap(),
                            ..options
                        },
                    ]);
                } else {
                    // Smoke particles (not realistic but to signal that the object is deorbiting)
                    self.queued_particles.push(options);
                }
            }
            if trail.len() >= ORBIT_OBJECT_TRAIL_LEN {
                trail.pop_back();
            }
            trail.push_front(*position);
        };
        for (id, position, velocity, radius, trail, &deorbiting) in query!(
            orbit.satellites,
            (
                id,
                &mut position,
                &velocity,
                &mut radius,
                &mut trail,
                &deorbiting
            )
        ) {
            move_object(
                Id::Satellite(id),
                position,
                velocity,
                radius,
                trail,
                deorbiting,
            );
        }
        for (id, position, velocity, radius, trail, &deorbiting) in query!(
            orbit.debris,
            (
                id,
                &mut position,
                &velocity,
                &mut radius,
                &mut trail,
                &deorbiting
            )
        ) {
            move_object(
                Id::Debris(id),
                position,
                velocity,
                radius,
                trail,
                deorbiting,
            );
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

        if collision_risk > CollisionRisk::Safe {
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
}
