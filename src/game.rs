mod ui;

pub use self::ui::GameUi;

use crate::{
    model::*,
    prelude::*,
    render::{GameRender, post::PostRender},
};

pub struct Game {
    context: Context,
    post: PostRender,
    render: GameRender,
    model: Model,
    ui_context: UiContext,
    ui: GameUi,
}

#[derive(Debug, Clone)]
pub enum GameAction {
    TheoreticResearch,
    LaunchSatellite,
}

impl Game {
    pub fn new(context: Context) -> Self {
        log::info!("Game started!");
        Self {
            context: context.clone(),
            post: PostRender::new(&context),
            render: GameRender::new(context.clone()),
            model: Model::new(&context.assets.config),
            ui_context: UiContext::new(context.clone()),
            ui: GameUi::new(&context),
        }
    }

    fn execute(&mut self, action: GameAction) {
        match action {
            GameAction::TheoreticResearch => {
                self.model.science += self.model.config.theoretic_research;
            }
            GameAction::LaunchSatellite => self.model.launch_satellite(true),
        }
    }
}

impl geng::State for Game {
    fn update(&mut self, delta_time: f64) {
        let delta_time = delta_time as f32;
        self.context.update(delta_time);
        self.ui_context.update(delta_time);
        let delta_time = Time::new(delta_time);
        self.model.update(delta_time);
    }

    fn handle_event(&mut self, event: geng::Event) {
        match event {
            geng::Event::Wheel { delta } => {
                self.ui_context.cursor.scroll += delta as f32;
            }
            geng::Event::CursorMove { position } => {
                self.ui_context.cursor.cursor_move(position.as_f32());
            }
            _ => (),
        }
    }

    fn draw(&mut self, screen_buffer: &mut ugli::Framebuffer) {
        let mut actions = Vec::new();
        self.ui.layout(
            &self.model,
            Aabb2::ZERO.extend_positive(screen_buffer.size().as_f32()),
            &mut self.ui_context,
            &mut actions,
        );
        self.ui_context.frame_end();
        for action in actions {
            self.execute(action);
        }

        ugli::clear(screen_buffer, Some(Rgba::BLACK), None, None);
        let framebuffer = &mut self
            .post
            .begin(screen_buffer.size(), crate::render::BACKGROUND_COLOR);

        self.render.draw(&self.model, framebuffer);
        self.render.draw_ui(&self.model, &self.ui, framebuffer);

        self.post.post_process(
            &self.context.get_options(),
            crate::render::post::PostVfx {
                time: self.model.real_time,
                crt: true,
            },
            screen_buffer,
        );
    }
}
