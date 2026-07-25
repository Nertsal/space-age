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

    fn draw_planet_layer(
        &self,
        model: &Model,
        planet: &Planet,
        framebuffer: &mut ugli::Framebuffer,
        layer: i32,
    ) {
        let planet_position = planet.position.to_cartesian();
        let planet_color = Color::try_from("#1e5c58").unwrap();
        let planet_transform =
            mat3::translate(planet_position) * mat3::scale_uniform(planet.radius * r32(2.0));

        let mut parameters = draw_parameters();
        if layer == 1 {
            parameters.depth_func = None;
            parameters.write_depth = false;
        }

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
                    u_layer: layer,
                },
                model.camera.uniforms(framebuffer.size().as_f32()),
            ),
            parameters,
        );
    }

    fn draw_rocket(
        &self,
        model: &Model,
        planet: &Planet,
        position: vec3<Coord>,
        payload: &Satellite,
        countdown: u32,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let planet_position = planet.position.to_cartesian();
        let camera = &model.camera;
        let rocket_texture = self.context.assets.sprites.rocket.as_ref();

        if position.z < Coord::ZERO && (position.xy() - planet_position).len() < planet.radius {
            return;
        }

        let direction = if countdown > 0 {
            position.xy() - planet_position
        } else {
            let target = payload.position.to_cartesian(planet_position);
            target.xy() - position.xy()
        };

        let angle =
            (Angle::atan2(direction.y, direction.x) - Angle::from_degrees(r32(90.0))).as_f32();

        let depth_scale = (Coord::ONE + position.z / planet.orbit.distance / r32(2.0))
            .clamp(Coord::ZERO, r32(2.0));
        let sprite_size = rocket_texture.size().as_f32() * 0.1 * depth_scale.as_f32();

        let quad = draw2d::TexturedQuad::colored(
            Aabb2::ZERO.extend_symmetric(sprite_size / 2.0),
            rocket_texture,
            Color::WHITE,
        )
        .transform(mat3::translate(position.xy().as_f32()) * mat3::rotate(angle));

        self.context
            .geng
            .draw2d()
            .draw2d(framebuffer, camera, &quad);
    }

    fn draw_planet(&mut self, model: &Model, planet: &Planet, framebuffer: &mut ugli::Framebuffer) {
        let camera = &model.camera;

        let planet_position = planet.position.to_cartesian();

        //planet base layer
        self.draw_planet_layer(model, planet, framebuffer, 0);

        // rockets that havent launched yet draw behind the clouds
        for (position, payload, countdown) in
            query!(planet.rockets, (&position, &payload, &countdown))
        {
            if *countdown > 0 {
                self.draw_rocket(model, planet, *position, payload, *countdown, framebuffer);
            }
        }

        // clouds
        self.draw_planet_layer(model, planet, framebuffer, 1);

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
        let satellite_comms_color = Color::try_from("#4E6BDE").unwrap();
        let satellite_debris_color = Color::try_from("#A5B452").unwrap();
        let debris_color = Color::try_from("#4B2F1B").unwrap();
        let satellite_active_color = Color::try_from("#1789FC").unwrap();
        let satellite_inactive_color = Color::try_from("#D72638").unwrap();
        for (pos, &radius, trail, lifetime, kind) in query!(
            planet.orbit.satellites,
            (&position, &visual_radius, &trail, &lifetime, &kind)
        ) {
            let color = match kind {
                SatelliteKind::Basic => satellite_color,
                SatelliteKind::Communication => satellite_comms_color,
                SatelliteKind::DebrisCleaner => satellite_debris_color,
            };
            let Some(scale) = draw_object(pos, radius, trail, color, framebuffer) else {
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

        // launched rockets
        for (position, payload, countdown) in
            query!(planet.rockets, (&position, &payload, &countdown))
        {
            if *countdown == 0 {
                self.draw_rocket(model, planet, *position, payload, *countdown, framebuffer);
            }
        }
    }

    pub fn draw_ui(&mut self, model: &Model, ui: &GameUi, framebuffer: &mut ugli::Framebuffer) {
        let camera = &geng::PixelPerfectCamera;
        let font = &self.context.assets.fonts.default;
        let sprites = &self.context.assets.sprites.ui;

        if model.abilities.contains(&Ability::CollisionAnalysis) {
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
            // Scientific Research
            let color = if ui.research_button.mouse_left.pressed.is_some() {
                Color::GRAY
            } else if ui.research_button.hovered {
                Color::try_from("#aaaaaa").unwrap()
            } else {
                Color::WHITE
            };
            self.ui.draw_texture(
                ui.research_button.position,
                &sprites.scientific_research,
                color,
                1.0,
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
            if !state.visible {
                continue;
            }

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
                    state.position.with_height(state.position.height() * t, 0.0),
                    Color::try_from("#333333").unwrap(),
                    camera,
                    framebuffer,
                );
            }

            let texture = match action {
                GameAction::Research(_) => None,
                GameAction::Action(action) => match action {
                    Action::TheoreticResearch => Some(&sprites.theoretic_research),
                    Action::Launch(kind) => None,
                    Action::Deorbit(_) => None,
                },
            };
            if let Some(texture) = texture {
                self.ui
                    .draw_texture(state.position, texture, color, 1.0, framebuffer);
            } else {
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

        // Info
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
        if model.abilities.contains(&Ability::RadarDebris) {
            self.util.draw_text_fit(
                format!("Debris: {}", model.debris()),
                ui.debris.position,
                font,
                TextRenderOptions::new(ui.pixel_scale * 10.0).align(vec2(0.0, 0.5)),
                camera,
                framebuffer,
            );
        }

        if let Some(id) = model.selected_object {
            // Selected object info
            let name = match id {
                InteractiveId::Satellite(id) => get!(model.planet.orbit.satellites, id, (&kind))
                    .map_or("Satellite".into(), |kind| format!("Satellite {:?}", kind)),
                InteractiveId::Debris(_) => "Debris".into(),
            };
            self.util.draw_text_fit(
                name,
                ui.selected.position,
                font,
                TextRenderOptions::new(ui.pixel_scale * 10.0),
                camera,
                framebuffer,
            );

            if ui.selected_deorbit.visible {
                let color = if ui.selected_deorbit.mouse_left.pressed.is_some() {
                    Color::GRAY
                } else if ui.selected_deorbit.hovered {
                    Color::try_from("#aaaaaa").unwrap()
                } else {
                    Color::try_from("#B61639").unwrap()
                };
                self.util.draw_text_fit(
                    "Deorbit",
                    ui.selected_deorbit.position,
                    font,
                    TextRenderOptions::new(ui.pixel_scale * 10.0).color(color),
                    camera,
                    framebuffer,
                );
            }
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
        self.ui
            .fill_quad_width(ui.research.position, width, Color::WHITE, framebuffer);
        self.ui
            .draw_outline(ui.research.position, width, Color::WHITE, framebuffer);
        // Title
        let title = ui
            .research
            .position
            .extend_symmetric(-vec2(6.0, 4.0) * ui.pixel_scale)
            .cut_top(10.0 * ui.pixel_scale);
        self.util.draw_text_fit(
            "Scientific Research",
            title,
            font,
            TextRenderOptions::new(title.height()).align(vec2(0.0, 0.5)),
            camera,
            framebuffer,
        );

        // Connections
        let connection_color = Color::try_from("#526985").unwrap();
        for item in &ui.research_items {
            if !item.state.visible {
                continue;
            }
            let Some(config) = model
                .config
                .research
                .items
                .iter()
                .find(|res| res.id == item.id)
            else {
                continue;
            };
            for &after in &config.after {
                let Some(other) = ui.research_items.iter().find(|item| item.id == after) else {
                    continue;
                };
                self.util.draw_segment(
                    framebuffer,
                    camera,
                    &draw2d::Segment::new(
                        Segment(item.state.position.center(), other.state.position.center()),
                        4.0 * ui.pixel_scale,
                        connection_color,
                    ),
                );
            }
        }

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

            let sprites = &self.context.assets.sprites.ui;
            let texture = model
                .get_research(item.id)
                .map_or(&sprites.research.orbit_program, |res| {
                    sprites.research.get_icon(&res.name)
                });
            let scale = item.state.position.size() / texture.size().as_f32();
            let scale = scale.x.min(scale.y);

            // Background fill
            // let radius = item.state.position.width() / 2.0;
            // self.context.geng.draw2d().circle(
            //     framebuffer,
            //     camera,
            //     item.state.position.center(),
            //     radius - 2.0,
            //     Color::WHITE,
            // );
            geng_utils::texture::DrawTexture::new(&sprites.research.fill)
                .pixel_perfect(
                    item.state.position.center(),
                    vec2(0.5, 0.5),
                    scale,
                    camera,
                    framebuffer,
                )
                .draw(camera, &self.context.geng, framebuffer);

            // Icon
            geng_utils::texture::DrawTexture::new(texture)
                .pixel_perfect(
                    item.state.position.center(),
                    vec2(0.5, 0.5),
                    scale,
                    camera,
                    framebuffer,
                )
                .colored(color)
                .draw(camera, &self.context.geng, framebuffer);
            // Outline
            geng_utils::texture::DrawTexture::new(&sprites.research.outline)
                .pixel_perfect(
                    item.state.position.center(),
                    vec2(0.5, 0.5),
                    scale,
                    camera,
                    framebuffer,
                )
                .colored(color)
                .draw(camera, &self.context.geng, framebuffer);

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
            let position = Aabb2::point(position)
                .extend_right(120.0 * ui.pixel_scale)
                .extend_down(75.0 * ui.pixel_scale);

            // Limit the window within the bounds
            // let bounds = ui.research.position;
            // if position.min.y < bounds.min.y {
            //     position = position.translate(vec2(0.0, bounds.min.y - position.min.y));
            // }

            // Boundary
            let width = ui.pixel_scale * 4.0;
            self.ui
                .fill_quad_width(position, width, Color::WHITE, framebuffer);
            self.ui
                .draw_outline(position, width, Color::WHITE, framebuffer);

            // Info
            let font_size = 10.0 * ui.pixel_scale;
            let options = TextRenderOptions::new(font_size)
                .color(Color::try_from("#F5F5F5").unwrap())
                .align(vec2(0.0, 0.5));

            let mut position = position.extend_uniform(-4.0 * ui.pixel_scale);
            let name = position.cut_top(font_size);
            self.util
                .draw_text_fit(&research.name, name, font, options, camera, framebuffer);
            let mut position = position.extend_symmetric(-vec2(6.0, 1.0) * ui.pixel_scale);

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
