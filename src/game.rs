mod ui;

pub use self::ui::GameUi;

use crate::{
    model::*,
    prelude::*,
    render::{GameRender, post::PostRender},
};

pub struct Game {
    ui_context: UiContext,
    context: Context,
    post: PostRender,
    render: GameRender,
    framebuffer_size: vec2<usize>,
    world_texture: ugli::Texture,
    world_depth: ugli::Renderbuffer<ugli::DepthComponent>,
    cursor: Cursor,

    model: Model,
    ui: GameUi,
}

struct Cursor {
    pub screen_pos: vec2<f32>,
    pub world_pos: vec2<Coord>,
}

#[derive(Debug, Clone)]
pub enum GameAction {
    Research(u64),
    Action(Action),
}

impl Game {
    pub fn new(context: Context, cheat: bool) -> Self {
        log::info!("Game started!");
        let mut game = Self {
            context: context.clone(),
            ui_context: UiContext::new(context.clone()),
            post: PostRender::new(&context),
            render: GameRender::new(context.clone()),
            framebuffer_size: vec2(1, 1),
            world_texture: geng_utils::texture::new_texture(context.geng.ugli(), vec2(1, 1)),
            world_depth: ugli::Renderbuffer::new(context.geng.ugli(), vec2(1, 1)),
            cursor: Cursor {
                screen_pos: vec2::ZERO,
                world_pos: vec2::ZERO,
            },

            model: Model::new(&context.assets.config),
            ui: GameUi::new(&context),
        };
        if cheat {
            game.model.science = 999999;
        }
        game
    }

    fn execute(&mut self, action: GameAction) {
        match action {
            GameAction::Research(id) => {
                self.model.research(id);
            }
            GameAction::Action(action) => {
                self.model.action(action);
            }
        }
    }

    fn click(&mut self) {
        self.model.select_hovered();
    }
}

impl geng::State for Game {
    fn update(&mut self, delta_time: f64) {
        let delta_time = delta_time as f32;
        self.context.update(delta_time);
        self.ui_context.update(delta_time);
        let delta_time = Time::new(delta_time);
        self.model.update_cursor(self.cursor.world_pos);
        self.model.update(delta_time);
    }

    fn handle_event(&mut self, event: geng::Event) {
        match event {
            geng::Event::Wheel { delta } => {
                self.ui_context.cursor.scroll += delta as f32;
            }
            geng::Event::CursorMove { position } => {
                self.ui_context.cursor.cursor_move(position.as_f32());
                self.cursor.screen_pos = position.as_f32();
                self.cursor.world_pos = self
                    .model
                    .camera
                    .screen_to_world(self.framebuffer_size.as_f32(), self.cursor.screen_pos)
                    .as_r32();
            }
            geng::Event::MousePress {
                button: geng::MouseButton::Left,
            } => {
                self.click();
            }
            geng::Event::KeyPress { key: geng::Key::F } => {
                if let Some(target) = self.model.selected_object {
                    self.model.action(Action::Deorbit(target));
                }
            }
            _ => (),
        }
    }

    fn draw(&mut self, screen_buffer: &mut ugli::Framebuffer) {
        self.framebuffer_size = screen_buffer.size();

        // Update texture size
        let world_size = (screen_buffer.size().as_f32()
            / crate::render::get_pixel_scale(screen_buffer.size()))
        .map(|x| x.round() as usize);
        if self.world_texture.size() != world_size {
            self.world_texture =
                ugli::Texture::new_with(self.context.geng.ugli(), world_size, |_| Rgba::BLACK);
            self.world_texture.set_filter(ugli::Filter::Nearest);
            self.world_depth = ugli::Renderbuffer::new(self.context.geng.ugli(), world_size);
        }

        // UI
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

        // Render world
        let world_buffer = &mut ugli::Framebuffer::new(
            self.context.geng.ugli(),
            ugli::ColorAttachment::Texture(&mut self.world_texture),
            ugli::DepthAttachment::Renderbuffer(&mut self.world_depth),
        );
        ugli::clear(world_buffer, Some(Color::BLACK), Some(1.0), None);
        self.render.draw(&self.model, world_buffer);
        geng_utils::texture::DrawTexture::new(&self.world_texture)
            .fit_screen(vec2(0.5, 0.5), framebuffer)
            .draw(&geng::PixelPerfectCamera, &self.context.geng, framebuffer);

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
