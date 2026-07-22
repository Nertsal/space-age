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

        // Font test
        let font = &*self.context.assets.fonts.default;
        let align = vec2(0.5, 0.5);

        // Regular rendering - Single line
        self.render.util.draw_text(
            "Simple text rendering",
            vec2(-7.0, 7.0),
            font,
            crate::render::util::TextRenderOptions::new(1.0).align(align),
            &self.model.camera,
            framebuffer,
        );
        self.context.geng.draw2d().circle(
            framebuffer,
            &self.model.camera,
            vec2(-7.0, 7.0),
            0.05,
            Color::RED,
        );
        // Regular rendering - Double line
        self.render.util.draw_text(
            "Multiline text rendering\nSecond line",
            vec2(7.0, 7.0),
            font,
            crate::render::util::TextRenderOptions::new(1.0).align(align),
            &self.model.camera,
            framebuffer,
        );
        self.context.geng.draw2d().circle(
            framebuffer,
            &self.model.camera,
            vec2(7.0, 7.0),
            0.05,
            Color::RED,
        );

        // Fit into target aabb - Single line
        let target = Aabb2::point(vec2(-7.0, 3.0)).extend_symmetric(vec2(5.0, 2.0) / 2.0);
        self.render
            .util
            .draw_quad(target, Color::BLUE, &self.model.camera, framebuffer);
        self.render.util.draw_text_fit(
            "This single line is fit into an AABB",
            target,
            font,
            crate::render::util::TextRenderOptions::new(1.0).align(align),
            &self.model.camera,
            framebuffer,
        );
        self.context.geng.draw2d().circle(
            framebuffer,
            &self.model.camera,
            target.center(),
            0.05,
            Color::RED,
        );
        // Fit into target aabb - Double line
        let target = Aabb2::point(vec2(7.0, 3.0)).extend_symmetric(vec2(5.0, 2.0) / 2.0);
        self.render
            .util
            .draw_quad(target, Color::BLUE, &self.model.camera, framebuffer);
        self.render.util.draw_text_fit(
            "This text is fit into an AABB\nWith multiple lines!",
            target,
            font,
            crate::render::util::TextRenderOptions::new(1.0).align(align),
            &self.model.camera,
            framebuffer,
        );
        self.context.geng.draw2d().circle(
            framebuffer,
            &self.model.camera,
            target.center(),
            0.05,
            Color::RED,
        );

        // Wrapped text rendering
        let target = Aabb2::point(vec2(0.0, -2.0)).extend_symmetric(vec2(8.0, 4.0) / 2.0);
        self.render
            .util
            .draw_quad(target, Color::BLUE, &self.model.camera, framebuffer);
        self.render.util.draw_text_wrap(
            "This text is automatically wrapped into multiple lines inside this AABB\nAnd manual line breaks also work!",
            target,
            font,
            crate::render::util::TextRenderOptions::new(0.6).align(align),
            &self.model.camera,
            framebuffer,
        );
        self.context.geng.draw2d().circle(
            framebuffer,
            &self.model.camera,
            target.center(),
            0.05,
            Color::RED,
        );

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
