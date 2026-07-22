pub mod post;
pub mod util;

use self::util::UtilRender;

use crate::{model::*, prelude::*};

pub const BACKGROUND_COLOR: Color = Color::BLACK;

#[allow(dead_code)]
pub struct GameRender {
    pub context: Context,
    pub util: UtilRender,
}

impl GameRender {
    pub fn new(context: Context) -> Self {
        Self {
            context: context.clone(),
            util: UtilRender::new(context.clone()),
        }
    }

    pub fn draw(&mut self, _model: &Model, _framebuffer: &mut ugli::Framebuffer) {}
}
