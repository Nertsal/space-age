use super::*;

use crate::{
    render::get_pixel_scale,
    ui::widget::{WidgetSfxConfig, WidgetState},
};

pub struct GameUi {
    pub pixel_scale: f32,
    pub science: WidgetState,
    pub actions: Vec<(WidgetState, GameAction)>,
}

impl GameUi {
    pub fn new(_context: &Context) -> Self {
        Self {
            pixel_scale: 1.0,
            science: WidgetState::new(),
            actions: vec![
                (
                    WidgetState::new().with_sfx(WidgetSfxConfig::hover_left()),
                    GameAction::TheoreticResearch,
                ),
                (
                    WidgetState::new().with_sfx(WidgetSfxConfig::hover_left()),
                    GameAction::LaunchSatellite,
                ),
            ],
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

        let mut panel = screen
            .align_aabb(vec2(screen.width() * 0.25, screen.height()), vec2(0.0, 0.5))
            .extend_uniform(-pixel_scale * 20.0);

        // Gold
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
    }
}
