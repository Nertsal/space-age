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
            radius: r32(0.3),
            trail: VecDeque::new(),
        });
    }
}
