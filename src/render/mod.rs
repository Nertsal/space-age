pub mod post;
pub mod ui;
pub mod util;

use self::{
    ui::UiRender,
    util::{TextRenderOptions, UtilRender},
};

use crate::{game::GameUi, model::*, prelude::*};

pub const BACKGROUND_COLOR: Color = Color::BLACK;

pub fn get_pixel_scale(framebuffer_size: vec2<usize>) -> f32 {
    const TARGET_SIZE: vec2<usize> = vec2(640, 360);
    let size = framebuffer_size.as_f32();
    let ratio = size / TARGET_SIZE.as_f32();
    ratio.x.min(ratio.y)
}

#[allow(dead_code)]
pub struct GameRender {
    pub context: Context,
    pub util: UtilRender,
    pub ui: UiRender,
}

impl GameRender {
    pub fn new(context: Context) -> Self {
        Self {
            context: context.clone(),
            util: UtilRender::new(context.clone()),
            ui: UiRender::new(context.clone()),
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
            let pos = pos.to_cartesian(planet_position);
            let scale = (Coord::ONE + pos.z / planet.orbit.distance / r32(2.0))
                .clamp(Coord::ZERO, r32(2.0)); // TODO: proper math instead of heuristic

            // Trail
            let trail = Chain::new(
                trail
                    .iter()
                    .map(|pos| pos.to_cartesian(planet_position))
                    .filter(|pos| pos.z > Coord::ZERO || pos.xy().len() > planet.radius)
                    .map(|pos| pos.xy().as_f32())
                    .collect(),
            );
            let trail = draw2d::Chain::new(
                trail,
                radius.as_f32() * 0.5 * scale.as_f32(),
                crate::util::with_alpha(Color::WHITE, 0.5),
                0,
            );
            self.util.draw_chain(framebuffer, camera, &trail);

            if pos.z < Coord::ZERO && pos.xy().len() < planet.radius {
                // Object is behind the planet
                continue;
            }

            // Object
            self.context.geng.draw2d().circle(
                framebuffer,
                camera,
                pos.xy().as_f32(),
                (*radius * scale).as_f32(),
                Color::try_from("#526985").unwrap(),
            );
        }
    }

    pub fn draw_ui(&mut self, model: &Model, ui: &GameUi, framebuffer: &mut ugli::Framebuffer) {
        let camera = &geng::PixelPerfectCamera;
        let font = &self.context.assets.fonts.default;

        self.util.draw_text_fit(
            format!("Science: {}", model.science),
            ui.science.position,
            font,
            TextRenderOptions::new(ui.pixel_scale * 10.0).align(vec2(0.0, 0.5)),
            camera,
            framebuffer,
        );

        for (state, action) in &ui.actions {
            let color = if state.mouse_left.pressed.is_some() {
                Color::GRAY
            } else if state.hovered {
                Color::try_from("#aaaaaa").unwrap()
            } else {
                Color::WHITE
            };
            self.util.draw_text_fit(
                format!("{:?}", action),
                state.position,
                font,
                TextRenderOptions::new(ui.pixel_scale * 10.0).color(color),
                camera,
                framebuffer,
            );
        }
    }
}
