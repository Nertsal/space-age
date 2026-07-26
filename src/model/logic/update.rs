use super::*;

impl Model {
    pub fn update(&mut self, delta_time: Time) {
        self.real_time += delta_time;

        self.hovered_rotation += Angle::from_degrees(r32(15.0) * delta_time);
        self.selected_rotation -= Angle::from_degrees(r32(15.0) * delta_time);

        // Update selection
        if let Some(id) = self.selected_object {
            let relevant = match id {
                InteractiveId::Satellite(id) => {
                    // TODO: better api for this in stecs
                    get!(self.planet.orbit.satellites, id, (&position)).is_some()
                }
                InteractiveId::Debris(id) => {
                    get!(self.planet.orbit.debris, id, (&position)).is_some()
                }
            };
            if !relevant {
                self.selected_object = None;
            }
        }

        // Theorycrafting
        // if auto_theory {
        //     self.theory_progress.change(delta_time);
        // }
        let planet_science_bonus = self.planet_science_bonus();
        while self.theory_progress > R32::ONE {
            let stat = self.get_stat(Stat::Theorycrafting);
            let gained = (self.config.theoretic_research.science as f32
                * (stat * planet_science_bonus).as_f32())
            .ceil() as Science;
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
        self.update_satellites(delta_time);
        self.update_rockets(delta_time);
        self.update_particles(delta_time);
    }

    fn planet_science_bonus(&self) -> R32 {
        if query!(self.planet.orbit.satellites, (&lifetime, &kind))
            .filter(|(lifetime, kind)| {
                lifetime.is_above_min() && matches!(kind, SatelliteKind::Communication)
            })
            .count()
            >= self.config.communications_bonus_requirement
        {
            R32::ONE + self.config.communications_bonus
        } else {
            R32::ONE
        }
    }

    pub fn update_satellites(&mut self, delta_time: Time) {
        let mut rng = thread_rng();

        // Update satellites production
        let planet_science_bonus = self.planet_science_bonus();
        let sat_eff = self.get_stat(Stat::SatelliteEfficiency);
        let longevity = self.get_stat(Stat::SatelliteLongevity);
        let planet = &mut self.planet;
        let orbit = &mut planet.orbit;
        for (kind, science_timer, lifetime, deorbiting) in query!(
            orbit.satellites,
            (&kind, &mut science_timer, &mut lifetime, &mut deorbiting)
        ) {
            if lifetime.is_min() {
                // This satellite is non-functioning
                if self.abilities.contains(&Ability::DeorbitAuto) {
                    if !*deorbiting && self.abilities.contains(&Ability::DeployAuto) {
                        planet.queued_launches.push(QueuedLaunch {
                            payment: true,
                            kind: *kind,
                        });
                    }
                    *deorbiting = true;
                }
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
                let gained = (config.science as f32 * (sat_eff * planet_science_bonus).as_f32())
                    .ceil() as Science;
                if gained > 0 {
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
        }

        // Satellite specific behavior -- Debris Cleaner
        let planet_position = planet.position.to_cartesian();
        for (kind, position, &radius, lifetime) in
            query!(orbit.satellites, (&kind, &position, &radius, &lifetime))
        {
            if lifetime.is_min() || !matches!(kind, SatelliteKind::DebrisCleaner) {
                continue;
            }
            let position = position.to_cartesian(planet_position);

            for (target_pos, &target_radius, deorbiting) in
                query!(orbit.debris, (&position, &radius, &mut deorbiting))
            {
                let target_pos = target_pos.to_cartesian(planet_position);
                let distance = (target_pos - position).len();
                if distance - radius - target_radius < self.config.debris_cleaner_range {
                    *deorbiting = true;
                }
            }
        }
    }

    pub fn update_rockets(&mut self, delta_time: Time) {
        // Setup queued rockets
        for launch in std::mem::take(&mut self.planet.queued_launches) {
            if !self.launch_satellite(launch.payment, launch.kind) {
                self.planet.queued_launches.push(launch);
            }
        }

        let planet = &mut self.planet;
        let planet_position = planet.position.to_cartesian();

        let speed = r32(8.0);
        let mut reached_target = Vec::new();

        for (id, position, countdown_time, countdown, payload, sfx) in query!(
            planet.rockets,
            (
                id,
                &mut position,
                &mut countdown_time,
                &mut countdown,
                &payload,
                &mut sfx,
            )
        ) {
            // has not launched yet
            if *countdown > 0 {
                let elapsed = self.real_time - *countdown_time;
                // countdown every 1 sec i guess
                if elapsed > r32(1.0) {
                    *countdown = countdown.saturating_sub(1);
                    *countdown_time = self.real_time;
                    if *countdown == 0 {
                        // launching now
                        let mut effect = self.context.sfx.play(&self.context.assets.sounds.rocket);
                        effect.set_volume(0.0);
                        effect.fade_to_volume(
                            self.context.get_options().volume.sfx() * 0.5,
                            time::Duration::from_secs_f64(0.5),
                        );
                        *sfx = Some(effect);
                    }
                }
                continue;
            }

            let target = payload.position.to_cartesian(planet_position);
            let offset = target - *position;
            let step = speed * delta_time;

            if offset.len() <= step {
                *position = target;
                if let Some(sfx) = sfx {
                    sfx.fade_out(time::Duration::from_secs_f64(0.5));
                }
                reached_target.push(id);
            } else {
                let direction = offset.normalize_or_zero();
                *position += direction * step;

                // Trail particles
                let options = SpawnParticles {
                    density: r32(1.0),
                    distribution: ParticleDistribution::Circle {
                        center: position.xy() - direction.xy() * r32(0.1),
                        radius: r32(0.3),
                    },
                    z: position.z - r32(0.01),
                    color: Color::try_from("#ADB3C2aa").unwrap(),
                    velocity: -direction.xy() * step * r32(0.5),
                    ..default()
                };
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
            }
        }

        let orbit = &mut planet.orbit;
        for id in reached_target {
            if let Some(rocket) = planet.rockets.remove(id) {
                orbit.satellites.insert(rocket.payload);
            }
        }
    }
}
