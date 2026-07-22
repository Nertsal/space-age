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
}

impl Game {
    pub fn new(context: Context) -> Self {
        log::info!("Game started!");
        Self {
            context: context.clone(),
            post: PostRender::new(&context),
            render: GameRender::new(context.clone()),
            model: Model::new(),
        }
    }
}

impl geng::State for Game {
    fn draw(&mut self, screen_buffer: &mut ugli::Framebuffer) {
        ugli::clear(screen_buffer, Some(Rgba::BLACK), None, None);
        let framebuffer = &mut self
            .post
            .begin(screen_buffer.size(), crate::render::BACKGROUND_COLOR);

        self.render.draw(&self.model, framebuffer);

        self.post.post_process(
            &self.context.get_options(),
            crate::render::post::PostVfx {
                time: self.model.real_time,
                crt: true,
            },
            screen_buffer,
        );
    }

    fn handle_event(&mut self, _event: geng::Event) {}

    fn update(&mut self, delta_time: f64) {
        let delta_time = delta_time as f32;
        self.context.update(delta_time);
        let delta_time = FloatTime::new(delta_time);
        self.model.update(delta_time);
    }
}
