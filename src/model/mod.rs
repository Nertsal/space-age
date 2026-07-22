mod logic;

use geng::prelude::*;

pub type FloatTime = R32;

pub struct Model {
    pub real_time: FloatTime,
    pub camera: Camera2d,
}

impl Model {
    pub fn new() -> Self {
        Self {
            real_time: FloatTime::ZERO,
            camera: Camera2d {
                center: vec2::ZERO,
                rotation: Angle::ZERO,
                fov: Camera2dFov::Cover {
                    width: 35.56,
                    height: 20.0,
                    scale: 1.0,
                },
            },
        }
    }
}
