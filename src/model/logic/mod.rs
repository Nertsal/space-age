use super::*;

use crate::util::random_angle;

impl Model {
    pub fn init(&mut self) {}

    pub fn update(&mut self, delta_time: Time) {
        self.real_time += delta_time;
        let mut rng = thread_rng();

        let orbit = &mut self.planet.orbit;
        // Update positions
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
        for science_timer in query!(orbit.satellites, (&mut science_timer)) {
            // NOTE: rng timer to desynchronise satelites so each one gives science at a different time
            science_timer.change(-delta_time - r32(rng.gen_range(-0.01..=0.01)));
            if science_timer.is_min() {
                science_timer.set_ratio(Time::ONE);
                self.science += 1;
            }
        }
    }

    pub fn launch_satellite(&mut self, pay_cost: bool) {
        if pay_cost {
            if self.science < 30 {
                return;
            }
            self.science -= 30;
        }

        let mut rng = thread_rng();

        let orbit = &mut self.planet.orbit;
        let position = SpherePos {
            distance: orbit.distance,
            polar: random_angle(&mut rng),
            azimuth: random_angle(&mut rng),
        };
        orbit.satellites.insert(Satellite {
            position,
            velocity: {
                let direction = random_angle::<Coord>(&mut rng);
                let speed = r32(rng.gen_range(0.5..0.7));
                let (polar, azimuth) = direction.sin_cos();
                // TODO: fix velocity calculation, the proper angle change is nonlinear
                SphereVelocity {
                    polar: Angle::from_radians(polar * speed),
                    azimuth: Angle::from_radians(azimuth * speed),
                }
            },
            radius: r32(0.3),
            trail: VecDeque::new(),
            science_timer: Bounded::new_max(r32(1.0)),
        });
    }
}
