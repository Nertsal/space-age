use super::*;

impl Model {
    pub fn update(&mut self, delta_time: Time) {
        self.real_time += delta_time;
        let mut rng = thread_rng();

        // Theorycrafting
        if self.theorizing {
            self.theory_progress.change(delta_time);

            if self.theory_progress.is_max() {
                let stat = self.get_stat(Stat::Theorycrafting);
                self.science += (self.config.theoretic_research.science as f32 * stat.as_f32())
                    .ceil() as Science;
                self.theory_progress.set_ratio(Time::ZERO);
                self.theorizing = false;
            }
        }

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
}
