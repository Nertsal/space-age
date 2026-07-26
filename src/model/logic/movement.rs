use super::*;

impl Model {
    pub fn movement(&mut self, delta_time: Time) {
        let mut rng = thread_rng();

        #[derive(Clone, Copy)]
        enum Id {
            Satellite(ArenaId),
            Debris(ArenaId),
        }

        let avoidance = self.get_stat(Stat::CollisionAvoidance).recip();
        let integrity = self.get_stat(Stat::SatelliteIntegrity);
        let collision_fragments =
            ((r32(5.0) - integrity + r32(1.0)).as_f32().ceil() as usize).max(2);

        let collision_risk = self.collision_risk();
        let planet = &mut self.planet;
        let planet_pos = planet.position.to_cartesian();
        let orbit = &mut planet.orbit;

        // Update positions
        let mut destroyed = Vec::new();
        let mut move_object =
            |id,
             position: &mut SpherePos,
             velocity: &SphereVelocity,
             radius: &mut Coord,
             trail: &mut VecDeque<SpherePos>,
             deorbiting: bool,
             burning: &mut Option<geng::SoundEffect>| {
                position.add_velocity(*velocity, delta_time);
                if deorbiting {
                    position.distance -= r32(1.0) * delta_time;
                    if position.distance < planet.radius {
                        destroyed.push(id);
                        if let Some(sfx) = burning {
                            sfx.fade_out(time::Duration::from_secs_f64(0.5));
                        }
                        return;
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
                        if burning.is_none() {
                            let mut effect =
                                self.context.sfx.play(&self.context.assets.sounds.burn);
                            effect.set_volume(0.0);
                            effect.fade_to_volume(
                                self.context.get_options().volume.sfx() * 0.2,
                                time::Duration::from_secs_f64(0.5),
                            );
                            *burning = Some(effect);
                        }
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
        for (id, position, velocity, rot, &rot_speed, radius, trail, &deorbiting, burning) in query!(
            orbit.satellites,
            (
                id,
                &mut position,
                &velocity,
                &mut rotation,
                &rotation_speed,
                &mut radius,
                &mut trail,
                &deorbiting,
                &mut burning,
            )
        ) {
            move_object(
                Id::Satellite(id),
                position,
                velocity,
                radius,
                trail,
                deorbiting,
                burning,
            );
            *rot += rot_speed * delta_time;
        }
        let debris_count = orbit.debris.ids().count();
        let deorbit_chance = if debris_count > self.config.debris_deorbit_threshold {
            self.config.debris_deorbit_chance * delta_time
        } else {
            R32::ZERO
        };
        for (id, position, velocity, radius, trail, deorbiting, burning) in query!(
            orbit.debris,
            (
                id,
                &mut position,
                &velocity,
                &mut radius,
                &mut trail,
                &mut deorbiting,
                &mut burning,
            )
        ) {
            move_object(
                Id::Debris(id),
                position,
                velocity,
                radius,
                trail,
                *deorbiting,
                burning,
            );
            if !*deorbiting
                && deorbit_chance > R32::ZERO
                && rng.gen_bool(deorbit_chance.as_f32().into())
            {
                *deorbiting = true;
            }
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
                    if distance < (rad_a + rad_b) * avoidance {
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
                    log::debug!("Collision of {:?}, fragments: {}", id, collision_fragments);
                    for _ in 0..collision_fragments {
                        orbit.debris.insert(Debris {
                            position: satellite.position,
                            velocity: action::random_orbit_velocity(satellite.position, &mut rng),
                            visual_radius: satellite.visual_radius / r32(2.0),
                            radius: satellite.radius / r32(4.0),
                            trail: trail.take().unwrap_or_default(),
                            deorbiting: rng.gen_bool(
                                self.config.collision_debris_deorbit_chance.as_f32().into(),
                            ),
                            burning: None,
                        });
                    }
                    // Explosion particles
                    let pos = satellite.position.to_cartesian(planet_pos);
                    let options = SpawnParticles {
                        density: r32(5.0),
                        distribution: ParticleDistribution::Circle {
                            center: pos.xy(),
                            radius: r32(1.5),
                        },
                        z: pos.z + r32(0.01),
                        color: Color::try_from("#E5302A").unwrap(),
                        ..default()
                    };
                    self.queued_particles.push(options);
                }
            }
        }
    }
}
