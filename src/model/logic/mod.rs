use super::*;

use crate::util::random_angle;

impl Model {
    pub fn init(&mut self) {
        for _ in 0..10 {
            self.launch_satellite(false);
        }
    }

    pub fn update(&mut self, delta_time: FloatTime) {
        self.real_time += delta_time;

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
        });
    }
}
