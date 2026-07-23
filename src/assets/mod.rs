mod config;
mod options;

pub use self::{config::*, options::*};

use geng::prelude::*;

#[derive(geng::asset::Load)]
pub struct LoadingAssets {
    #[load(path = "fonts/default.ttf")]
    pub font: geng::Font,
    #[load(path = "shaders/crt.glsl")]
    pub shader_crt: Rc<ugli::Program>,
}

#[derive(geng::asset::Load)]
pub struct Assets {
    pub shaders: Shaders,
    pub fonts: Fonts,
    pub sounds: Sounds,
    pub sprites: Sprites,
    pub config: Config,
}

#[derive(geng::asset::Load)]
pub struct Fonts {
    pub default: Rc<geng::Font>,
}

#[derive(geng::asset::Load)]
pub struct Shaders {
    pub crt: Rc<ugli::Program>,
    pub texture: Rc<ugli::Program>,
    pub splitcut: Rc<ugli::Program>,
    pub solid: Rc<ugli::Program>,
}

#[derive(geng::asset::Load)]
pub struct Sounds {
    pub ui_click: Rc<geng::Sound>,
    pub ui_hover: Rc<geng::Sound>,
}

#[derive(geng::asset::Load)]
pub struct Sprites {
    pub border: Rc<ugli::Texture>,
    pub fill: Rc<ugli::Texture>,
}
