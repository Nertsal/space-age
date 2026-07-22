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

        let planet_position = planet.position.to_cartesian();
        self.context.geng.draw2d().circle(
            framebuffer,
            camera,
            planet_position.as_f32(),
            planet.radius.as_f32(),
            Color::try_from("#1e5c58").unwrap(),
        );

        for (pos, radius, trail) in query!(
            [planet.orbit.satellites, planet.orbit.debris],
            (&position, &radius, &trail)
        ) {
            // Trail

            // Object
            self.context.geng.draw2d().circle(
                framebuffer,
                camera,
                pos.to_cartesian(planet_position).xy().as_f32(),
                radius.as_f32(),
                Color::try_from("#526985").unwrap(),
            );
        }
    }
}
