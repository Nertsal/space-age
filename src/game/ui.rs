use super::*;

use crate::{
    render::get_pixel_scale,
    ui::widget::{WidgetSfxConfig, WidgetState},
};

use geng_utils::interpolation::SecondOrderState;

pub struct GameUi {
    pub pixel_scale: f32,
    pub screen: Aabb2<f32>,
    pub research_button: WidgetState,
    pub science: WidgetState,
    pub actions: Vec<(WidgetState, GameAction)>,

    pub research: WidgetState,
    pub research_close: WidgetState,
    pub research_camera: Camera2d,
    pub research_fov: SecondOrderState<vec2<f32>>,
    pub research_items: Vec<ResearchItemWidget>,

    pub info: WidgetState,
    pub active_satellites: WidgetState,
    pub inactive_satellites: WidgetState,
    pub debris: WidgetState,

    pub selected: WidgetState,
    pub selected_lifetime: WidgetState,
    pub selected_deorbit: WidgetState,

    pub collision_risk: WidgetState,
}

pub struct ResearchItemWidget {
    pub id: u64,
    /// World position.
    pub position: Aabb2<f32>,
    pub state: WidgetState,
}

impl GameUi {
    pub fn new(context: &Context) -> Self {
        let mut ui = Self {
            pixel_scale: 1.0,
            screen: Aabb2::ZERO.extend_positive(vec2(1600.0, 900.0)),
            research_button: WidgetState::new().with_sfx(WidgetSfxConfig::hover_left()),
            science: WidgetState::new(),
            actions: vec![
                (
                    WidgetState::new().with_sfx(WidgetSfxConfig::hover_left()),
                    GameAction::Action(Action::TheoreticResearch),
                ),
                (
                    WidgetState::new().with_sfx(WidgetSfxConfig::hover_left()),
                    GameAction::Action(Action::Launch(SatelliteKind::Basic)),
                ),
                (
                    WidgetState::new().with_sfx(WidgetSfxConfig::hover_left()),
                    GameAction::Action(Action::Launch(SatelliteKind::Communication)),
                ),
                (
                    WidgetState::new().with_sfx(WidgetSfxConfig::hover_left()),
                    GameAction::Action(Action::Launch(SatelliteKind::DebrisCleaner)),
                ),
            ],

            research: WidgetState::new().hidden(),
            research_close: WidgetState::new().with_sfx(WidgetSfxConfig::hover_left()),
            research_camera: Camera2d {
                center: vec2::ZERO,
                rotation: Angle::ZERO,
                fov: Camera2dFov::Cover {
                    width: 7.0,
                    height: 7.0,
                    scale: 1.0,
                },
            },
            research_fov: SecondOrderState::new(3.0, 1.0, 0.0, vec2::splat(3.0)),
            research_items: Vec::new(),

            info: WidgetState::new(),
            active_satellites: WidgetState::new(),
            inactive_satellites: WidgetState::new(),
            debris: WidgetState::new(),

            selected: WidgetState::new(),
            selected_lifetime: WidgetState::new(),
            selected_deorbit: WidgetState::new(),

            collision_risk: WidgetState::new(),
        };
        ui.populate_research(&context.assets.config);
        ui
    }

    fn populate_research(&mut self, config: &Config) {
        self.research_items.clear();

        for item in &config.research.items {
            self.research_items.push(ResearchItemWidget {
                position: Aabb2::point(item.pos.as_f32()).extend_symmetric(vec2::splat(0.4) / 2.0),
                state: WidgetState::new().with_sfx(WidgetSfxConfig::hover_left()),
                id: item.id,
            });
        }
    }

    pub fn layout(
        &mut self,
        model: &Model,
        screen: Aabb2<f32>,
        context: &mut UiContext,
        actions: &mut Vec<GameAction>,
    ) {
        // let layout_size = screen.height() * 0.05;
        let pixel_scale = get_pixel_scale(screen.size().map(|x| x as usize));
        self.pixel_scale = pixel_scale;
        self.screen = screen;

        let panel = screen.extend_symmetric(-vec2(50.0, 40.0) * pixel_scale);
        let mut panel = panel.align_aabb(vec2(pixel_scale * 48.0, panel.height()), vec2(0.0, 0.5));

        // Research
        let research = panel.cut_top(pixel_scale * 48.0);
        self.research_button.update(research, context);
        if self.research_button.mouse_left.clicked {
            self.research.toggle_visibility();
        }

        panel.cut_top(pixel_scale * 20.0);

        // Science
        let science = panel.cut_top(pixel_scale * 20.0);
        self.science.update(science, context);

        // Actions
        let rows = panel
            .clone()
            .cut_top(48.0 * pixel_scale)
            .stack(vec2(0.0, -48.0 * pixel_scale), self.actions.len());
        for ((state, action), row) in itertools::izip![&mut self.actions, rows] {
            if let GameAction::Action(action) = action {
                state.set_visibility(model.abilities.contains(&action.ability()));
            }
            state.update(row, context);
            if state.mouse_left.clicked {
                actions.push(action.clone());
            }
        }

        // Research window
        if self.research.visible {
            let research = screen.extend_symmetric(-vec2(120.0, 40.0) * pixel_scale);
            self.research.update(research, context);
            self.research_fov.update(context.delta_time);
            self.research_camera.fov = Camera2dFov::Cover {
                width: self.research_fov.current.x,
                height: self.research_fov.current.y,
                scale: 1.0,
            };

            // Close button
            let close = research
                .extend_symmetric(-vec2(3.0, 3.0) * pixel_scale)
                .align_aabb(vec2(10.0, 10.0) * pixel_scale, vec2(1.0, 1.0));
            self.research_close.update(close, context);
            if self.research_close.mouse_left.clicked {
                self.research.hide();
            }

            let mut bounds = Aabb2::ZERO;
            for item in &mut self.research_items {
                let position = item.position.map_bounds(|p| {
                    self.research_camera
                        .world_to_screen(screen.size(), p)
                        .unwrap_either()
                });
                item.state.update(position, context);

                let state = model.get_research_state(item.id);

                item.state.set_visibility(matches!(
                    state,
                    ResearchState::Researched | ResearchState::Available { .. }
                ));
                if item.state.visible {
                    bounds = crate::util::extend_cover(bounds, item.position);
                }

                if let ResearchState::Available { .. } = state
                    && item.state.mouse_left.clicked
                {
                    actions.push(GameAction::Research(item.id));
                }
            }

            let view_area = vec2(
                (-bounds.min.x).max(bounds.max.x),
                (-bounds.min.y).max(bounds.max.y),
            );
            self.research_fov.target =
                view_area * 2.0 * (screen.size() / self.research.position.size())
                    + vec2::splat(1.5);
        }

        // Top panel
        let collision_risk = screen
            .align_aabb(vec2(150.0, 30.0) * pixel_scale, vec2(0.5, 1.0))
            .translate(vec2(0.0, -15.0) * pixel_scale);
        self.collision_risk.update(collision_risk, context);

        // Right panel
        let mut panel = screen
            .align_aabb(
                vec2(screen.width() * 0.25, screen.height() * 0.7),
                vec2(1.0, 0.5),
            )
            .extend_uniform(-pixel_scale * 20.0);

        self.info.update(
            panel
                .extend_uniform(pixel_scale * 12.0)
                .extend_up(pixel_scale * 5.0),
            context,
        );

        // Radar
        self.active_satellites
            .update(panel.cut_top(pixel_scale * 20.0), context);
        self.inactive_satellites
            .update(panel.cut_top(pixel_scale * 20.0), context);
        self.debris
            .update(panel.cut_top(pixel_scale * 20.0), context);

        // Selected object
        panel.cut_top(pixel_scale * 50.0);
        self.selected
            .update(panel.cut_top(pixel_scale * 20.0), context);
        if let Some(InteractiveId::Satellite(_)) = model.selected_object {
            self.selected_lifetime
                .update(panel.cut_top(pixel_scale * 20.0), context);
        }
        self.selected_deorbit
            .update(panel.cut_top(pixel_scale * 20.0), context);
        self.selected_deorbit
            .set_visibility(model.abilities.contains(&Ability::Deorbit));
        if self.selected_deorbit.mouse_left.clicked
            && let Some(target) = model.selected_object
        {
            actions.push(GameAction::Action(Action::Deorbit(target)));
        }
    }
}
