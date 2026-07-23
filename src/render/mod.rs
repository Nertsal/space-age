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
            let mut trail = draw2d::Chain::new(
                trail,
                radius.as_f32() * 0.5 * scale.as_f32(),
                crate::util::with_alpha(Color::WHITE, 0.5),
                0,
            );
            let len = trail.vertices.len();
            for (i, v) in trail.vertices.iter_mut().enumerate() {
                let t = 1.0 - (i + 1) as f32 / len as f32;
                v.a_color.a *= geng_utils::interpolation::smoothstep(t);
            }
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

        {
            let color = if ui.research_button.mouse_left.pressed.is_some() {
                Color::GRAY
            } else if ui.research_button.hovered {
                Color::try_from("#aaaaaa").unwrap()
            } else {
                Color::WHITE
            };
            self.util.draw_text_fit(
                "Scientific Research",
                ui.research_button.position,
                font,
                TextRenderOptions::new(ui.pixel_scale * 10.0).color(color),
                camera,
                framebuffer,
            );
        }

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

        self.draw_ui_research(model, ui, framebuffer);
    }

    fn draw_ui_research(
        &mut self,
        model: &Model,
        ui: &GameUi,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        if !ui.research.visible {
            return;
        }

        let camera = &geng::PixelPerfectCamera;
        let font = &self.context.assets.fonts.default;

        // Background
        let width = ui.pixel_scale * 4.0;
        self.ui.fill_quad_width(
            ui.research.position,
            width,
            Color::try_from("#1E1B18").unwrap(),
            framebuffer,
        );
        self.ui.draw_outline(
            ui.research.position,
            width,
            Color::try_from("#0A2463").unwrap(),
            framebuffer,
        );

        // Items
        let color_researched = Color::try_from("#F2F3D9").unwrap();
        let color_available = Color::try_from("#3E92CC").unwrap();
        let color_expensive = Color::try_from("#E36987").unwrap();
        let color_locked = Color::try_from("#D8315B").unwrap();

        let mut hovered = None;
        for item in &ui.research_items {
            let state = model.get_research_state(item.id);
            let color = match state {
                ResearchState::Researched => color_researched,
                ResearchState::Available { cost } => {
                    if model.science >= cost {
                        color_available
                    } else {
                        color_expensive
                    }
                }
                ResearchState::Locked => color_locked,
            };
            self.context.geng.draw2d().circle(
                framebuffer,
                camera,
                item.state.position.center(),
                item.state.position.width() / 2.0,
                color,
            );

            if item.state.hovered {
                hovered = Some((item.id, item.state.position));
            }
        }

        // Hover info
        if let Some((id, position)) = hovered
            && let Some(research) = model
                .config
                .research
                .items
                .iter()
                .find(|item| item.id == id)
        {
            let position = position.top_right() + vec2(10.0, 10.0) * ui.pixel_scale;
            let mut position = Aabb2::point(position)
                .extend_right(120.0 * ui.pixel_scale)
                .extend_down(75.0 * ui.pixel_scale);
            let bounds = ui.research.position;
            if position.min.y < bounds.min.y {
                position = position.translate(vec2(0.0, bounds.min.y - position.min.y));
            }

            // Boundary
            let width = ui.pixel_scale * 4.0;
            self.ui.fill_quad_width(
                position,
                width,
                Color::try_from("#1E1B18").unwrap(),
                framebuffer,
            );
            self.ui.draw_outline(
                position,
                width,
                Color::try_from("#0A2463").unwrap(),
                framebuffer,
            );

            // Info
            let font_size = 10.0 * ui.pixel_scale;
            let options = TextRenderOptions::new(font_size)
                .color(Color::try_from("#F5F5F5").unwrap())
                .align(vec2(0.0, 0.5));

            let mut position = position.extend_uniform(-8.0 * ui.pixel_scale);
            let name = position.cut_top(font_size);
            self.util
                .draw_text_fit(&research.name, name, font, options, camera, framebuffer);

            if !matches!(model.get_research_state(id), ResearchState::Researched) {
                let cost = position.cut_top(font_size);
                self.util.draw_text_fit(
                    format!("Cost: {}", research.cost),
                    cost,
                    font,
                    options,
                    camera,
                    framebuffer,
                );
            }
            self.util.draw_text_wrap(
                &research.description,
                position,
                font,
                options,
                camera,
                framebuffer,
            );
        }
    }
}
