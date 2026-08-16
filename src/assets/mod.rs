mod config;
mod localization;
mod options;

pub use self::{config::*, localization::*, options::*};

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
    pub localization: Localization,
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
    pub collision: Rc<geng::Sound>,
    pub research_complete: Rc<geng::Sound>,
    #[load(list = "0..=7")]
    pub research: Vec<Rc<geng::Sound>>,

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
    #[load(options(filter = "ugli::Filter::Nearest"))]
    pub crashlanding: Rc<ugli::Texture>,
    #[load(options(filter = "ugli::Filter::Nearest"))]
    pub structural_integrity: Rc<ugli::Texture>,
    #[load(options(filter = "ugli::Filter::Nearest"))]
    pub theorycrafting: Rc<ugli::Texture>,
    #[load(options(filter = "ugli::Filter::Nearest"))]
    pub engineering: Rc<ugli::Texture>,
    #[load(options(filter = "ugli::Filter::Nearest"))]
    pub communications: Rc<ugli::Texture>,
    #[load(options(filter = "ugli::Filter::Nearest"))]
    pub data_compression: Rc<ugli::Texture>,
    #[load(options(filter = "ugli::Filter::Nearest"))]
    pub debris_cleaning: Rc<ugli::Texture>,
    #[load(options(filter = "ugli::Filter::Nearest"))]
    pub orbit_observation: Rc<ugli::Texture>,
    #[load(options(filter = "ugli::Filter::Nearest"))]
    pub automation: Rc<ugli::Texture>,
    #[load(options(filter = "ugli::Filter::Nearest"))]
    pub collision_avoidance: Rc<ugli::Texture>,
}

impl SpritesResearch {
    pub fn get_icon(&self, name: &str) -> &Rc<ugli::Texture> {
        match name {
            "research.orbit_program" => &self.orbit_program,
            "research.crashlanding" => &self.crashlanding,
            "research.theorycrafting1" => &self.theorycrafting,
            "research.theorycrafting2" => &self.theorycrafting,
            "research.theorycrafting3" => &self.theorycrafting,
            "research.engineering1" => &self.engineering,
            "research.engineering2" => &self.engineering,
            "research.structural_integrity1" => &self.structural_integrity,
            "research.structural_integrity2" => &self.structural_integrity,
            "research.data_compression1" => &self.data_compression,
            "research.data_compression2" => &self.data_compression,
            "research.data_compression3" => &self.data_compression,
            "research.communications" => &self.communications,
            "research.orbit observation" => &self.orbit_observation,
            "research.debris_cleaning" => &self.debris_cleaning,
            "research.collision_avoidance1" => &self.collision_avoidance,
            "research.collision_avoidance2" => &self.collision_avoidance,
            "research.automation1" => &self.automation,
            "research.automation2" => &self.automation,
            "research.orbit_observation" => &self.orbit_observation,
            _ => &self.orbit_program,
        }
    }
}
