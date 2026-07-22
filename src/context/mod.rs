mod music;
mod sfx;

use crate::{
    assets::{Assets, Options},
    context::{music::MusicManager, sfx::SfxManager},
};

use geng::prelude::*;
use time::Duration;

#[derive(Clone)]
pub struct Context {
    pub geng: Geng,
    pub assets: Rc<Assets>,
    pub music: Rc<MusicManager>,
    pub sfx: Rc<SfxManager>,
    options: Rc<RefCell<Options>>,
}

impl Context {
    pub fn new(geng: Geng, assets: Rc<Assets>) -> Self {
        let options = Rc::new(RefCell::new(
            preferences::load(crate::OPTIONS_STORAGE).unwrap_or_default(),
        ));
        Self {
            music: Rc::new(MusicManager::new(geng.clone())),
            sfx: Rc::new(SfxManager::new(geng.clone(), options.clone())),
            geng,
            assets,
            options,
        }
    }

    pub fn update(&self, delta_time: f32) {
        self.music.update(delta_time);
    }

    pub fn get_options(&self) -> Options {
        self.options.borrow().clone()
    }

    #[allow(dead_code)]
    pub fn set_options(&self, options: Options) {
        let old = self.options.borrow();
        if *old != options {
            drop(old);
            self.force_set_options(options);
        }
    }

    fn force_set_options(&self, options: Options) {
        let mut old = self.options.borrow_mut();

        self.music.set_volume(options.volume.music());

        preferences::save(crate::OPTIONS_STORAGE, &options);
        *old = options;
    }
}
