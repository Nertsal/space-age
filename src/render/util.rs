#![allow(dead_code)]

use super::*;

use geng::Font;

#[derive(Debug, Clone, Copy)]
pub struct TextRenderOptions {
    pub size: f32,
    pub align: vec2<f32>,
    pub color: Color,
    pub rotation: Angle,
    /// Extra space between lines, measured relative to the font size.
    pub line_spacing: f32,
}

impl TextRenderOptions {
    pub fn new(size: f32) -> Self {
        Self { size, ..default() }
    }

    pub fn size(self, size: f32) -> Self {
        Self { size, ..self }
    }

    pub fn align(self, align: vec2<f32>) -> Self {
        Self { align, ..self }
    }

    pub fn color(self, color: Color) -> Self {
        Self { color, ..self }
    }
}

impl Default for TextRenderOptions {
    fn default() -> Self {
        Self {
            size: 1.0,
            align: vec2::splat(0.5),
            color: Color::WHITE,
            rotation: Angle::ZERO,
            line_spacing: 0.0,
        }
    }
}

pub struct UtilRender {
    context: Context,
    pub unit_quad: ugli::VertexBuffer<draw2d::TexturedVertex>,
}

impl UtilRender {
    pub fn new(context: Context) -> Self {
        Self {
            unit_quad: geng_utils::geometry::unit_quad_geometry(context.geng.ugli()),
            context,
        }
    }

    pub fn draw_quad(
        &self,
        quad: Aabb2<f32>,
        color: Color,
        camera: &impl geng::AbstractCamera2d,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        self.context
            .geng
            .draw2d()
            .quad(framebuffer, camera, quad, color);
    }

    // pub fn draw_nine_slice_pp(
    //     &self,
    //     pos: Aabb2<f32>,
    //     color: Color,
    //     texture: &ugli::Texture,
    //     scale: f32,
    //     camera: &impl geng::AbstractCamera2d,
    //     framebuffer: &mut ugli::Framebuffer,
    // ) {
    //     let pos = geng_utils::pixel::pixel_perfect_aabb(
    //         pos.center(),
    //         vec2(0.5, 0.5),
    //         texture.size().map(|x| (x as f32 * scale).round() as usize),
    //         camera,
    //         framebuffer.size().as_f32(),
    //     );
    //     self.draw_nine_slice(pos, color, texture, scale, camera, framebuffer)
    // }

    pub fn draw_nine_slice(
        &self,
        pos: Aabb2<f32>,
        color: Color,
        texture: &ugli::Texture,
        scale: f32,
        camera: &impl geng::AbstractCamera2d,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let whole = Aabb2::ZERO.extend_positive(vec2::splat(1.0));

        // TODO: configurable
        let mid = Aabb2 {
            min: vec2(0.3, 0.3),
            max: vec2(0.7, 0.7),
        };

        let size = mid.min * texture.size().as_f32() * scale;
        let size = vec2(size.x.min(pos.width()), size.y.min(pos.height()));

        let tl = Aabb2::from_corners(mid.top_left(), whole.top_left());
        let tm = Aabb2::from_corners(mid.top_left(), vec2(mid.max.x, whole.max.y));
        let tr = Aabb2::from_corners(mid.top_right(), whole.top_right());
        let rm = Aabb2::from_corners(mid.top_right(), vec2(whole.max.x, mid.min.y));
        let br = Aabb2::from_corners(mid.bottom_right(), whole.bottom_right());
        let bm = Aabb2::from_corners(mid.bottom_right(), vec2(mid.min.x, whole.min.y));
        let bl = Aabb2::from_corners(mid.bottom_left(), whole.bottom_left());
        let lm = Aabb2::from_corners(mid.bottom_left(), vec2(whole.min.x, mid.max.y));

        let slices: Vec<draw2d::TexturedVertex> = [tl, tm, tr, rm, br, bm, bl, lm, mid]
            .into_iter()
            .flat_map(|slice| {
                let [a, b, c, d] = slice.corners().map(|a_vt| {
                    let a_pos = vec2(
                        if a_vt.x == mid.min.x {
                            pos.min.x + size.x
                        } else if a_vt.x == mid.max.x {
                            pos.max.x - size.x
                        } else {
                            pos.min.x + pos.width() * a_vt.x
                        },
                        if a_vt.y == mid.min.y {
                            pos.min.y + size.y
                        } else if a_vt.y == mid.max.y {
                            pos.max.y - size.y
                        } else {
                            pos.min.y + pos.height() * a_vt.y
                        },
                    );
                    draw2d::TexturedVertex {
                        a_pos,
                        a_color: Color::WHITE,
                        a_vt,
                    }
                });
                [a, b, c, a, c, d]
            })
            .collect();
        let slices = ugli::VertexBuffer::new_dynamic(self.context.geng.ugli(), slices);

        ugli::draw(
            framebuffer,
            &self.context.assets.shaders.texture,
            ugli::DrawMode::Triangles,
            &slices,
            (
                ugli::uniforms! {
                    u_model_matrix: mat3::identity(),
                    u_color: color,
                    u_texture: texture,
                },
                camera.uniforms(framebuffer.size().as_f32()),
            ),
            ugli::DrawParameters {
                blend_mode: Some(ugli::BlendMode::straight_alpha()),
                ..default()
            },
        );

        // self.geng
        //     .draw2d()
        //     .textured_quad(framebuffer, camera, pos, texture, color);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw_nine_slice_splitcut(
        &self,
        pos: Aabb2<f32>,
        color: Color,
        cut_x: f32,
        texture_left: &ugli::Texture,
        texture_right: &ugli::Texture,
        scale: f32,
        camera: &impl geng::AbstractCamera2d,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let whole = Aabb2::ZERO.extend_positive(vec2::splat(1.0));

        // TODO: configurable
        let mid = Aabb2 {
            min: vec2(0.3, 0.3),
            max: vec2(0.7, 0.7),
        };

        let size = mid.min * texture_left.size().as_f32() * scale;
        let size = vec2(size.x.min(pos.width()), size.y.min(pos.height()));

        let tl = Aabb2::from_corners(mid.top_left(), whole.top_left());
        let tm = Aabb2::from_corners(mid.top_left(), vec2(mid.max.x, whole.max.y));
        let tr = Aabb2::from_corners(mid.top_right(), whole.top_right());
        let rm = Aabb2::from_corners(mid.top_right(), vec2(whole.max.x, mid.min.y));
        let br = Aabb2::from_corners(mid.bottom_right(), whole.bottom_right());
        let bm = Aabb2::from_corners(mid.bottom_right(), vec2(mid.min.x, whole.min.y));
        let bl = Aabb2::from_corners(mid.bottom_left(), whole.bottom_left());
        let lm = Aabb2::from_corners(mid.bottom_left(), vec2(whole.min.x, mid.max.y));

        let slices: Vec<draw2d::TexturedVertex> = [tl, tm, tr, rm, br, bm, bl, lm, mid]
            .into_iter()
            .flat_map(|slice| {
                let [a, b, c, d] = slice.corners().map(|a_vt| {
                    let a_pos = vec2(
                        if a_vt.x == mid.min.x {
                            pos.min.x + size.x
                        } else if a_vt.x == mid.max.x {
                            pos.max.x - size.x
                        } else {
                            pos.min.x + pos.width() * a_vt.x
                        },
                        if a_vt.y == mid.min.y {
                            pos.min.y + size.y
                        } else if a_vt.y == mid.max.y {
                            pos.max.y - size.y
                        } else {
                            pos.min.y + pos.height() * a_vt.y
                        },
                    );
                    draw2d::TexturedVertex {
                        a_pos,
                        a_color: Color::WHITE,
                        a_vt,
                    }
                });
                [a, b, c, a, c, d]
            })
            .collect();
        let slices = ugli::VertexBuffer::new_dynamic(self.context.geng.ugli(), slices);

        ugli::draw(
            framebuffer,
            &self.context.assets.shaders.splitcut,
            ugli::DrawMode::Triangles,
            &slices,
            (
                ugli::uniforms! {
                    u_model_matrix: mat3::identity(),
                    u_color: color,
                    u_cut_x: cut_x,
                    u_texture_left: texture_left,
                    u_texture_right: texture_right,
                },
                camera.uniforms(framebuffer.size().as_f32()),
            ),
            ugli::DrawParameters {
                blend_mode: Some(ugli::BlendMode::straight_alpha()),
                ..default()
            },
        );

        // self.geng
        //     .draw2d()
        //     .textured_quad(framebuffer, camera, pos, texture, color);
    }

    pub fn draw_text(
        &self,
        text: impl AsRef<str>,
        position: vec2<impl Float>,
        font: &Font,
        options: TextRenderOptions,
        camera: &impl geng::AbstractCamera2d,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        self.draw_text_with(
            &[text],
            position,
            font,
            options,
            ugli::DrawParameters {
                blend_mode: Some(ugli::BlendMode::straight_alpha()),
                ..default()
            },
            camera,
            framebuffer,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw_text_with(
        &self,
        text_lines: &[impl AsRef<str>],
        position: vec2<impl Float>,
        font: &Font,
        mut options: TextRenderOptions,
        params: ugli::DrawParameters,
        camera: &impl geng::AbstractCamera2d,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let framebuffer_size = framebuffer.size().as_f32();

        let position = position.map(Float::as_f32);
        let position = camera
            .world_to_screen(framebuffer_size, position)
            .unwrap_either();

        let scale = camera
            .world_to_screen(
                framebuffer_size,
                vec2::splat(std::f32::consts::FRAC_1_SQRT_2),
            )
            .unwrap_either()
            - camera
                .world_to_screen(framebuffer_size, vec2::ZERO)
                .unwrap_either();
        options.size *= scale.len();
        let font_size = options.size;

        let line_height = font_size;
        let line_spacing = options.line_spacing * font_size;
        let lines: usize = text_lines
            .iter()
            .map(|line| line.as_ref().lines().count())
            .sum();
        let height_align = line_height * lines.saturating_sub(1) as f32
            + line_spacing * lines.saturating_sub(2) as f32;
        let total_vertical_alignment = height_align * (1.0 - options.align.y);

        let mut position = position + vec2(0.0, total_vertical_alignment);
        for line in text_lines.iter().flat_map(|line| line.as_ref().lines()) {
            let measure = font
                .measure(line, vec2::splat(geng::TextAlign::CENTER))
                .unwrap_or(Aabb2::ZERO);
            let size = measure.size() * font_size;

            // default alignment is (0.0, 1.0)
            let align = vec2(options.align.x, 1.0 - options.align.y);
            let descent = -font.descender() * font_size;
            let ascent = font.ascender() * font_size;
            let align = vec2(size.x, descent - ascent) * align;

            let transform = mat3::translate(position)
                * mat3::rotate(options.rotation)
                * mat3::translate(-align)
                * mat3::scale_uniform(font_size);

            font.draw(
                framebuffer,
                &geng::PixelPerfectCamera,
                line,
                vec2(0.0, 0.5).map(geng::TextAlign),
                transform,
                options.color,
            );
            position.y -= line_height + line_spacing;
        }
    }

    pub fn draw_text_fit(
        &self,
        text: impl AsRef<str>,
        target: Aabb2<f32>,
        font: &Font,
        mut options: TextRenderOptions,
        camera: &impl geng::AbstractCamera2d,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let text = text.as_ref();
        let measure = font
            .measure(text, vec2::splat(geng::TextAlign::CENTER))
            .unwrap_or(Aabb2::ZERO);

        let max_size = target.size();
        let right = vec2(max_size.x, 0.0).rotate(options.rotation).x;
        let left = vec2(0.0, max_size.y).rotate(options.rotation).x;
        let width = if left.signum() != right.signum() {
            left.abs() + right.abs()
        } else {
            left.abs().max(right.abs())
        };

        let max_height = max_size.y * 0.9;
        let max_width = width; // * 0.85; // Leave some space TODO: move into a parameter or smth
        let max_size = max_width / measure.width();
        let size = options.size.min(max_size).min(max_height);

        options.size = size;

        self.draw_text_with(
            &[text],
            target.align_pos(options.align),
            font,
            options,
            ugli::DrawParameters {
                blend_mode: Some(ugli::BlendMode::straight_alpha()),
                ..default()
            },
            camera,
            framebuffer,
        );
    }

    pub fn draw_text_wrap(
        &self,
        text: impl AsRef<str>,
        target: Aabb2<f32>,
        font: &Font,
        options: TextRenderOptions,
        camera: &impl geng::AbstractCamera2d,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let text = text.as_ref();
        let max_size = target.size();
        let right = vec2(max_size.x, 0.0).rotate(options.rotation).x;
        let left = vec2(0.0, max_size.y).rotate(options.rotation).x;
        let width = if left.signum() != right.signum() {
            left.abs() + right.abs()
        } else {
            left.abs().max(right.abs())
        };

        let max_width = width;
        let lines = crate::util::wrap_text(font, text, max_width / options.size);
        self.draw_text_with(
            &lines,
            target.align_pos(options.align),
            font,
            options,
            ugli::DrawParameters {
                blend_mode: Some(ugli::BlendMode::straight_alpha()),
                ..default()
            },
            camera,
            framebuffer,
        );
    }

    pub fn draw_chain(
        &self,
        framebuffer: &mut ugli::Framebuffer,
        camera: &impl geng::AbstractCamera2d,
        chain: &draw2d::Chain,
    ) {
        let framebuffer_size = framebuffer.size();
        ugli::draw(
            framebuffer,
            &self.context.assets.shaders.solid,
            ugli::DrawMode::Triangles,
            &ugli::VertexBuffer::new_dynamic(self.context.geng.ugli(), chain.vertices.clone()),
            (
                ugli::uniforms! {
                    u_color: Rgba::WHITE,
                    u_framebuffer_size: framebuffer_size,
                    u_model_matrix: chain.transform,
                },
                camera.uniforms(framebuffer_size.map(|x| x as f32)),
            ),
            ugli::DrawParameters {
                blend_mode: Some(ugli::BlendMode::straight_alpha()),
                ..Default::default()
            },
        );
    }

    pub fn draw_segment(
        &self,
        framebuffer: &mut ugli::Framebuffer,
        camera: &impl geng::AbstractCamera2d,
        segment: &draw2d::Segment,
    ) {
        let framebuffer_size = framebuffer.size();
        ugli::draw(
            framebuffer,
            &self.context.assets.shaders.solid,
            ugli::DrawMode::TriangleFan,
            &ugli::VertexBuffer::new_dynamic(self.context.geng.ugli(), segment.vertices.clone()),
            (
                ugli::uniforms! {
                    u_color: Rgba::WHITE,
                    u_framebuffer_size: framebuffer_size,
                    u_model_matrix: segment.transform,
                },
                camera.uniforms(framebuffer_size.map(|x| x as f32)),
            ),
            ugli::DrawParameters {
                blend_mode: None,
                ..Default::default()
            },
        );
    }
}
