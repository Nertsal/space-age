use super::*;

impl Model {
    pub fn update(&mut self, delta_time: FloatTime) {
        self.real_time += delta_time;
    }
}
