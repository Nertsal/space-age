use super::*;

use crate::{
    render::get_pixel_scale,
    ui::widget::{WidgetSfxConfig, WidgetState},
};

pub struct GameUi {
    pub pixel_scale: f32,
    pub screen: Aabb2<f32>,
    pub research_button: WidgetState,
    pub science: WidgetState,
    pub actions: Vec<(WidgetState, GameAction)>,

    pub research: WidgetState,
    pub research_camera: Camera2d,
    pub research_items: Vec<ResearchItemWidget>,
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
            research_button: WidgetState::new(),
            science: WidgetState::new(),
            actions: vec![
                (
                    WidgetState::new().with_sfx(WidgetSfxConfig::hover_left()),
                    GameAction::Action(Action::TheoreticResearch),
                ),
                (
                    WidgetState::new().with_sfx(WidgetSfxConfig::hover_left()),
                    GameAction::Action(Action::Launch(SatelliteType::Basic)),
                ),
            ],

            research: WidgetState::new().hidden(),
            research_camera: Camera2d {
                center: vec2::ZERO,
                rotation: Angle::ZERO,
                fov: Camera2dFov::Cover {
                    width: 10.0,
                    height: 10.0,
                    scale: 1.0,
                },
            },
            research_items: Vec::new(),
        };
        ui.populate_research(&context.assets.config);
        ui
    }

    fn populate_research(&mut self, config: &Config) {
        self.research_items.clear();

        let mut position = vec2::ZERO;
        for item in &config.research.items {
            self.research_items.push(ResearchItemWidget {
                position: Aabb2::point(position).extend_symmetric(vec2(1.0, 1.0) / 2.0),
                state: WidgetState::new(),
                id: item.id,
            });

            position += vec2(0.0, 1.5);
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

        let mut panel = screen
            .align_aabb(vec2(screen.width() * 0.25, screen.height()), vec2(0.0, 0.5))
            .extend_uniform(-pixel_scale * 20.0);

        // Research
        let research = panel.cut_top(pixel_scale * 20.0);
        self.research_button.update(research, context);
        if self.research_button.mouse_left.clicked {
            self.research.toggle_visibility();
        }

        // Science
        let science = panel.cut_top(pixel_scale * 20.0);
        self.science.update(science, context);

        // Items
        let rows = panel
            .clone()
            .cut_top(25.0 * pixel_scale)
            .stack(vec2(0.0, -25.0 * pixel_scale), self.actions.len());
        for ((state, action), row) in itertools::izip![&mut self.actions, rows] {
            state.update(row, context);
            if state.mouse_left.clicked {
                actions.push(action.clone());
            }
        }

        // Research window
        if self.research.visible {
            let research = screen.extend_uniform(-50.0 * pixel_scale);
            self.research.update(research, context);

            for item in &mut self.research_items {
                let position = item.position.map_bounds(|p| {
                    self.research_camera
                        .world_to_screen(screen.size(), p)
                        .unwrap_either()
                });
                item.state.update(position, context);
                if item.state.mouse_left.clicked
                    && let ResearchState::Available { .. } = model.get_research_state(item.id)
                {
                    actions.push(GameAction::Research(item.id));
                }
            }
        }
    }
}
