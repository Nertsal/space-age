pub mod post;
pub mod ui;
pub mod util;

use self::{
    ui::UiRender,
    util::{TextRenderOptions, UtilRender},
};

use crate::{
    game::{GameAction, GameUi},
    model::*,
    prelude::*,
};

pub const BACKGROUND_COLOR: Color = Color::BLACK;

pub fn get_pixel_scale(framebuffer_size: vec2<usize>) -> f32 {
    const TARGET_SIZE: vec2<usize> = vec2(640, 360);
    let size = framebuffer_size.as_f32();
    let ratio = size / TARGET_SIZE.as_f32();
    ratio.x.min(ratio.y)
}

pub fn draw_parameters() -> ugli::DrawParameters {
    ugli::DrawParameters {
        blend_mode: Some(ugli::BlendMode::straight_alpha()),
        depth_func: Some(ugli::DepthFunc::Less),
        ..default()
    }
}

pub fn z_depth(z: Coord) -> f32 {
    -z.as_f32() / 50.0
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

        let planet = &model.planet;
        let planet_pos = planet.position.to_cartesian();
        let is_behind_planet = |pos: vec3<Coord>| -> bool {
            pos.z < Coord::ZERO && (pos.xy() - planet_pos).len() < planet.radius
        };

        // Particles
        #[derive(ugli::Vertex)]
        struct ParticleInstance {
            pub i_color: Color,
            pub i_model_matrix: mat3<f32>,
            pub i_z: f32,
        }
        let instances: Vec<_> = query!(model.particles, (&color, &position, &radius, &lifetime))
            .filter_map(|(&color, &position, &radius, lifetime)| {
                if is_behind_planet(position) {
                    return None;
                }
                let scale = (Coord::ONE + position.z / planet.orbit.distance / r32(2.0))
                    .clamp(Coord::ZERO, r32(2.0)); // TODO: proper math instead of heuristic
                let t = lifetime.get_ratio().as_f32().sqrt();
                let color = crate::util::with_alpha(color, t);
                let transform = mat3::translate(position.xy().as_f32())
                    * mat3::scale_uniform(radius.as_f32() * scale.as_f32() * t);
                Some(ParticleInstance {
                    i_color: color,
                    i_model_matrix: transform,
                    i_z: z_depth(position.z),
                })
            })
            .collect();
        let instances = ugli::VertexBuffer::new_dynamic(self.context.geng.ugli(), instances);
        ugli::draw(
            framebuffer,
            &self.context.assets.shaders.particles,
            ugli::DrawMode::TriangleFan,
            ugli::instanced(&self.util.unit_quad, &instances),
            (
                ugli::uniforms! {},
                model.camera.uniforms(framebuffer.size().as_f32()),
            ),
            draw_parameters(),
        );

        // Texticles
        for (text, &position, &size, &color, lifetime) in query!(
            model.texticles,
            (&text, &position, &size, &color, &lifetime)
        ) {
            let font = &self.context.assets.fonts.default;
            let t = lifetime.get_ratio().sqrt();
            let size = size * t;
            self.util.draw_text(
                text,
                position.xy().as_f32(),
                font,
                TextRenderOptions::new(size.as_f32())
                    .color(crate::util::with_alpha(color, t.as_f32())),
                &model.camera,
                framebuffer,
            );
        }

        // Selection
        if let Some(id) = model.hovered_object
            && model.hovered_object != model.selected_object
        {
            self.draw_selection(
                model,
                id,
                Color::try_from("#ADD7F6").unwrap(),
                model.hovered_rotation,
                framebuffer,
            );
        }
        if let Some(id) = model.selected_object {
            self.draw_selection(
                model,
                id,
                Color::try_from("#87BFFF").unwrap(),
                model.selected_rotation,
                framebuffer,
            );
        }
    }

    fn draw_selection(
        &mut self,
        model: &Model,
        id: InteractiveId,
        mut color: Color,
        rotation: Angle<Coord>,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let planet = &model.planet;
        let orbit = &planet.orbit;
        let Some((pos, &radius, &deorbiting)) = (match id {
            InteractiveId::Satellite(id) => get!(
                orbit.satellites,
                id,
                (&position, &visual_radius, &deorbiting)
            ),
            InteractiveId::Debris(id) => {
                get!(orbit.debris, id, (&position, &visual_radius, &deorbiting))
            }
        }) else {
            return;
        };

        if deorbiting {
            color = Color::try_from("#B61639").unwrap();
        }

        let planet_pos = planet.position.to_cartesian();
        let pos = pos.to_cartesian(planet_pos);
        let pixel_scale = 0.1;
        let pos = Aabb2::point(pos.xy()).extend_uniform(radius + r32(pixel_scale * 4.0));
        self.util.draw_nine_slice(
            pos.as_f32(),
            color,
            &self.context.assets.sprites.selected,
            pixel_scale,
            rotation.as_f32(),
            &model.camera,
            framebuffer,
        );
    }

    fn draw_planet(&mut self, model: &Model, planet: &Planet, framebuffer: &mut ugli::Framebuffer) {
        let camera = &model.camera;

        // Planet
        let planet_position = planet.position.to_cartesian();
        let planet_color = Color::try_from("#1e5c58").unwrap();
        let planet_transform =
            mat3::translate(planet_position) * mat3::scale_uniform(planet.radius * r32(2.0));

        ugli::draw(
            framebuffer,
            &self.context.assets.shaders.planet,
            ugli::DrawMode::TriangleFan,
            &self.util.unit_quad,
            (
                ugli::uniforms! {
                    u_model_matrix: planet_transform.as_f32(),
                    u_color: planet_color,
                    u_framebuffer_size: framebuffer.size().as_f32(),
                    u_time: model.real_time.as_f32(),
                },
                camera.uniforms(framebuffer.size().as_f32()),
            ),
            draw_parameters(),
        );

        // Orbit
        let draw_object = |pos: &SpherePos,
                           radius: Coord,
                           trail: &VecDeque<SpherePos>,
                           color: Color,
                           framebuffer: &mut ugli::Framebuffer<'_>|
         -> Option<Coord> {
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
                return None;
            }

            // Object
            self.context.geng.draw2d().circle(
                framebuffer,
                camera,
                pos.xy().as_f32(),
                (radius * scale).as_f32(),
                color,
            );

            Some(scale)
        };

        let satellite_color = Color::try_from("#526985").unwrap();
        let debris_color = Color::try_from("#4B2F1B").unwrap();
        let satellite_active_color = Color::try_from("#1789FC").unwrap();
        let satellite_inactive_color = Color::try_from("#D72638").unwrap();
        for (pos, &radius, trail, lifetime) in query!(
            planet.orbit.satellites,
            (&position, &visual_radius, &trail, &lifetime)
        ) {
            let Some(scale) = draw_object(pos, radius, trail, satellite_color, framebuffer) else {
                continue;
            };
            let blink_pos = pos.to_cartesian(planet_position).xy()
                + vec2::splat(r32(std::f32::consts::FRAC_1_SQRT_2)) * r32(0.8) * radius * scale;
            let blink_color = if lifetime.is_above_min() {
                satellite_active_color
            } else {
                satellite_inactive_color
            };
            self.context.geng.draw2d().circle(
                framebuffer,
                camera,
                blink_pos.as_f32(),
                (radius * scale).as_f32() * 0.25,
                blink_color,
            );
        }
        for (pos, &radius, trail) in
            query!(planet.orbit.debris, (&position, &visual_radius, &trail))
        {
            draw_object(pos, radius, trail, debris_color, framebuffer);
        }
    }

    pub fn draw_ui(&mut self, model: &Model, ui: &GameUi, framebuffer: &mut ugli::Framebuffer) {
        let camera = &geng::PixelPerfectCamera;
        let font = &self.context.assets.fonts.default;

        {
            // Collision risk
            let mut risk = ui.collision_risk.position;
            let left = risk.split_left(0.5);
            self.util.draw_text_fit(
                "Collision Risk: ",
                left,
                font,
                TextRenderOptions::new(ui.pixel_scale * 15.0).align(vec2(1.0, 0.0)),
                camera,
                framebuffer,
            );
            let collision_risk = model.collision_risk();
            let color = match collision_risk {
                CollisionRisk::Safe => Color::try_from("#2AFC98").unwrap(),
                CollisionRisk::Caution => Color::try_from("#CFF137").unwrap(),
                CollisionRisk::Moderate => Color::try_from("#EF8A17").unwrap(),
                CollisionRisk::Severe => Color::try_from("#B61639").unwrap(),
            };
            self.util.draw_text_fit(
                format!("  {:?}", collision_risk),
                risk,
                font,
                TextRenderOptions::new(ui.pixel_scale * 15.0)
                    .align(vec2(0.0, 0.0))
                    .color(color),
                camera,
                framebuffer,
            );
        }

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

            // Action progress
            if let GameAction::Action(Action::TheoreticResearch) = action
                && model.theory_progress > R32::ZERO
            {
                let t = model.theory_progress.as_f32().clamp(0.0, 1.0);
                self.util.draw_quad(
                    state.position.with_width(state.position.width() * t, 0.0),
                    Color::try_from("#333333").unwrap(),
                    camera,
                    framebuffer,
                );
            }

            self.util.draw_text_fit(
                format!("{:?}", action),
                state.position,
                font,
                TextRenderOptions::new(ui.pixel_scale * 10.0).color(color),
                camera,
                framebuffer,
            );
        }

        self.util.draw_text_fit(
            format!("Active Satellites: {}", model.active_satellites()),
            ui.active_satellites.position,
            font,
            TextRenderOptions::new(ui.pixel_scale * 10.0).align(vec2(0.0, 0.5)),
            camera,
            framebuffer,
        );
        self.util.draw_text_fit(
            format!("Dysfunctional Satellites: {}", model.inactive_satellites()),
            ui.inactive_satellites.position,
            font,
            TextRenderOptions::new(ui.pixel_scale * 10.0).align(vec2(0.0, 0.5)),
            camera,
            framebuffer,
        );
        self.util.draw_text_fit(
            format!("Debris: {}", model.debris()),
            ui.debris.position,
            font,
            TextRenderOptions::new(ui.pixel_scale * 10.0).align(vec2(0.0, 0.5)),
            camera,
            framebuffer,
        );

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
            if !item.state.visible {
                continue;
            }

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
