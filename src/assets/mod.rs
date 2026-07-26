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

pub struct Fonts {
    pub default: Rc<geng::Font>,
}

impl geng::asset::Load for Fonts {
    type Options = ();

    fn load(
        manager: &geng::asset::Manager,
        path: &std::path::Path,
        &(): &Self::Options,
    ) -> geng::asset::Future<Self> {
        let manager = manager.clone();
        let path = path.to_owned();
        async move {
            Ok(Self {
                default: manager
                    .load_with(
                        path.join("default.ttf"),
                        &geng::font::Options {
                            antialias: false,
                            distance_mode: geng::font::DistanceMode::Max,
                            ..default()
                        },
                    )
                    .await?,
            })
        }
        .boxed_local()
    }

    const DEFAULT_EXT: Option<&'static str> = None;
}

#[derive(geng::asset::Load)]
pub struct Shaders {
    pub crt: Rc<ugli::Program>,
    pub texture: Rc<ugli::Program>,
    pub splitcut: Rc<ugli::Program>,
    pub solid: Rc<ugli::Program>,
    pub planet: Rc<ugli::Program>,
    pub particles: Rc<ugli::Program>,
}

#[derive(geng::asset::Load)]
pub struct Sounds {
    pub music: Rc<geng::Sound>,

    pub rocket: Rc<geng::Sound>,
    pub burn: Rc<geng::Sound>,

    pub ui_click: Rc<geng::Sound>,
    pub ui_hover: Rc<geng::Sound>,
}

#[derive(geng::asset::Load)]
pub struct Sprites {
    pub ui: SpritesUi,

    #[load(options(filter = "ugli::Filter::Nearest"))]
    pub satellite: Rc<ugli::Texture>,
    #[load(options(filter = "ugli::Filter::Nearest"))]
    pub selected: Rc<ugli::Texture>,
    #[load(options(filter = "ugli::Filter::Nearest"))]
    pub rocket: Rc<ugli::Texture>,
    #[load(list = "1..=5")]
    pub countdown: Vec<Rc<ugli::Texture>>,
}

#[derive(geng::asset::Load)]
pub struct SpritesUi {
    #[load(options(filter = "ugli::Filter::Nearest"))]
    pub border: Rc<ugli::Texture>,
    #[load(options(filter = "ugli::Filter::Nearest"))]
    pub fill: Rc<ugli::Texture>,
    #[load(options(filter = "ugli::Filter::Nearest"))]
    pub close: Rc<ugli::Texture>,
    #[load(options(filter = "ugli::Filter::Nearest"))]
    pub exclamation: Rc<ugli::Texture>,

    #[load(options(filter = "ugli::Filter::Nearest"))]
    pub scientific_research: Rc<ugli::Texture>,
    #[load(options(filter = "ugli::Filter::Nearest"))]
    pub theoretic_research: Rc<ugli::Texture>,
    #[load(options(filter = "ugli::Filter::Nearest"))]
    pub launch: Rc<ugli::Texture>,

    pub research: SpritesResearch,
}

#[derive(geng::asset::Load)]
pub struct SpritesResearch {
    #[load(options(filter = "ugli::Filter::Nearest"))]
    pub outline: Rc<ugli::Texture>,
    #[load(options(filter = "ugli::Filter::Nearest"))]
    pub fill: Rc<ugli::Texture>,

    #[load(options(filter = "ugli::Filter::Nearest"))]
    pub orbit_program: Rc<ugli::Texture>,
}

impl SpritesResearch {
    pub fn get_icon(&self, name: &str) -> &Rc<ugli::Texture> {
        match name {
            "Orbit Program" => &self.orbit_program,
            "Crashlanding" => &self.orbit_program,
            "Theorycrafting I" => &self.orbit_program,
            "Theorycrafting II" => &self.orbit_program,
            "Engineering I" => &self.orbit_program,
            "Engineering II" => &self.orbit_program,
            "Structural Integrity I" => &self.orbit_program,
            "Structural Integrity II" => &self.orbit_program,
            "Data Compression I" => &self.orbit_program,
            "Data Compression II" => &self.orbit_program,
            "Communications" => &self.orbit_program,
            "Orbit Observation" => &self.orbit_program,
            "Collision Risk Analysis" => &self.orbit_program,
            "Debris Cleaning" => &self.orbit_program,
            "Collision Avoidance" => &self.orbit_program,
            "Automation I" => &self.orbit_program,
            "Automation II" => &self.orbit_program,
            _ => &self.orbit_program,
        }
    }
}
