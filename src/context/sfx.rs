use super::*;

pub struct SfxManager {
    geng: Geng,
    options: Rc<RefCell<Options>>,
}

impl SfxManager {
    pub fn new(geng: Geng, options: Rc<RefCell<Options>>) -> Self {
        Self { geng, options }
    }

    pub fn play(&self, sfx: &geng::Sound) -> geng::SoundEffect {
        self.play_volume(sfx, 1.0)
    }

    pub fn play_volume(&self, sfx: &geng::Sound, volume: f32) -> geng::SoundEffect {
        let options = self.options.borrow();
        let mut effect = sfx.effect(self.geng.audio().default_type());
        effect.set_volume(options.volume.sfx() * volume);
        effect.play();
        effect
    }

    pub fn play_random_speed(&self, sfx: &geng::Sound, volume: f32) -> geng::SoundEffect {
        let mut effect = self.play_volume(sfx, volume);
        let speed = thread_rng().gen_range(0.9..=1.1);
        effect.set_speed(speed);
        effect
    }
}
