use super::*;

impl Model {
    pub fn update_particles(&mut self, delta_time: Time) {
        // Spawn from queue
        for spawn in std::mem::take(&mut self.queued_particles) {
            self.spawn_particles(spawn);
        }

        // Move particles
        let mut old = Vec::new();
        for (id, position, velocity, lifetime) in query!(
            self.particles,
            (id, &mut position, &velocity, &mut lifetime)
        ) {
            *position += *velocity * delta_time;
            lifetime.change(-delta_time);
            if lifetime.is_min() {
                old.push(id);
            }
        }
        for id in old {
            self.particles.remove(id);
        }
    }

    pub fn spawn_particles(&mut self, options: SpawnParticles) {
        let mut rng = thread_rng();
        let new_particles = options
            .distribution
            .sample(&mut rng, options.density)
            .into_iter()
            .map(move |position| {
                let velocity = rng.gen_circle(options.velocity, r32(0.2));
                let radius = rng.gen_range(options.size.clone());
                let lifetime = rng.gen_range(options.lifetime.clone());
                Particle {
                    position: position.extend(options.z),
                    radius,
                    velocity: velocity.extend(Coord::ZERO),
                    color: options.color,
                    lifetime: Bounded::new_max(lifetime),
                }
            });
        for particle in new_particles {
            self.particles.insert(particle);
        }
    }
}
