pub mod post;
pub mod util;

use self::util::UtilRender;

use crate::{model::*, prelude::*};

pub const BACKGROUND_COLOR: Color = Color::BLACK;

#[allow(dead_code)]
pub struct GameRender {
    pub context: Context,
    pub util: UtilRender,
}

impl GameRender {
    pub fn new(context: Context) -> Self {
        Self {
            context: context.clone(),
            util: UtilRender::new(context.clone()),
        }
    }

    pub fn draw(&mut self, model: &Model, framebuffer: &mut ugli::Framebuffer) {
        self.draw_planet(model, &model.planet, framebuffer);
    }

    fn draw_planet(&mut self, model: &Model, planet: &Planet, framebuffer: &mut ugli::Framebuffer) {
        let camera = &model.camera;
        self.context.geng.draw2d().circle(
            framebuffer,
            camera,
            planet.position.to_cartesian().as_f32(),
            planet.radius.as_f32(),
            Color::try_from("#b56452").unwrap(),
        );
    }
}
