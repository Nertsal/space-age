use super::*;

use crate::util::random_angle;

impl Model {
    pub fn init(&mut self) {
        for _ in 0..10 {
            self.launch_satellite();
        }
    }

    pub fn update(&mut self, delta_time: FloatTime) {
        self.real_time += delta_time;

        let orbit = &mut self.planet.orbit;
        for (position, velocity) in
            query!([orbit.satellites, orbit.debris], (&mut position, &velocity))
        {
            position.polar += velocity.polar * delta_time;
            position.azimuth += velocity.azimuth * delta_time;
        }
    }

    pub fn launch_satellite(&mut self) {
        let mut rng = thread_rng();

        let orbit = &mut self.planet.orbit;
        orbit.satellites.insert(Satellite {
            position: SpherePos {
                distance: orbit.distance,
                polar: random_angle(&mut rng),
                azimuth: random_angle(&mut rng),
            },
            velocity: {
                let direction = random_angle::<Coord>(&mut rng);
                let speed = r32(rng.gen_range(2.0..3.0));
                let (polar, azimuth) = direction.sin_cos();
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
