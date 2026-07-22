#![allow(dead_code)]

use geng::{Font, prelude::*};

/// If `f` if `true`, returns `1`, else `0`.
pub fn one<T: UNum>(f: bool) -> T {
    if f { T::ONE } else { T::ZERO }
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
