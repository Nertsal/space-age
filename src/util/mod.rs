#![allow(dead_code)]

use geng::{Font, prelude::*};

/// If `f` if `true`, returns `1`, else `0`.
pub fn one<T: UNum>(f: bool) -> T {
    if f { T::ONE } else { T::ZERO }
}

pub fn random_angle<T: Float>(rng: &mut impl Rng) -> Angle<T> {
    let radians = rng.gen_range(0.0..=std::f32::consts::TAU);
    Angle::from_radians(T::from_f32(radians))
}

pub fn with_alpha(mut color: Rgba<f32>, alpha: f32) -> Rgba<f32> {
    color.a *= alpha;
    color
}

/// Wrap text based on the relative target max width of the text.
pub fn wrap_text(font: &Font, text: &str, target_width: f32) -> Vec<String> {
    let mut lines = Vec::new();
    for source_line in text.lines() {
        let mut line = String::new();
        for word in source_line.split_whitespace() {
            if line.is_empty() {
                line += word;
                continue;
            }
            if font
                .measure(
                    &(line.clone() + " " + word),
                    vec2::splat(geng::TextAlign::CENTER),
                )
                .unwrap_or(Aabb2::ZERO)
                .width()
                > target_width
            {
                lines.push(line);
                line = word.to_string();
            } else {
                line += " ";
                line += word;
            }
        }
        if !line.is_empty() {
            lines.push(line);
        }
    }
    lines
}

pub fn extend_point(s: Aabb2<f32>, point: vec2<f32>) -> Aabb2<f32> {
    Aabb2 {
        min: vec2(s.min.x.min(point.x), s.min.y.min(point.y)),
        max: vec2(s.max.x.max(point.x), s.max.y.max(point.y)),
    }
}

pub fn extend_cover(s: Aabb2<f32>, other: Aabb2<f32>) -> Aabb2<f32> {
    extend_point(extend_point(s, other.min), other.max)
}
