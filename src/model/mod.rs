mod logic;

use crate::prelude::*;

use stecs::prelude::*;

pub type Arena<V> =
    stecs::storage::slotmap::SlotMap<stecs::storage::slotmap::slotmap::DefaultKey, V>;

pub type FloatTime = R32;
pub type Coord = R32;

pub struct Model {
    pub real_time: FloatTime,
    pub camera: Camera2d,

    pub planet: Planet,
}

impl Model {
    pub fn new() -> Self {
        Self {
            real_time: FloatTime::ZERO,
            camera: Camera2d {
                center: vec2::ZERO,
                rotation: Angle::ZERO,
                fov: Camera2dFov::Cover {
                    width: 50.0,
                    height: 50.0,
                    scale: 1.0,
                },
            },

            planet: Planet::new(),
        }
    }
}

pub struct Planet {
    /// Position of the planet in the solar system.
    pub position: PolarPos,
    pub radius: Coord,
    pub orbit: Orbit,
}

impl Planet {
    pub fn new() -> Self {
        Self {
            position: PolarPos {
                distance: Coord::ZERO,
                angle: Angle::ZERO,
            },
            radius: r32(10.0),
            orbit: Orbit::default(),
        }
    }
}

#[derive(Default)]
pub struct Orbit {
    pub satellites: StructOf<Arena<Satellite>>,
    pub debris: StructOf<Arena<Debris>>,
}

#[derive(SplitFields)]
pub struct Satellite {
    pub position: SpherePos,
}

#[derive(SplitFields)]
pub struct Debris {
    #[split(nested)]
    pub position: SpherePos,
}

pub struct PolarPos {
    pub distance: Coord,
    pub angle: Angle<Coord>,
}

impl PolarPos {
    pub fn to_cartesian(&self) -> vec2<Coord> {
        self.angle.unit_vec() * self.distance
    }
}

#[derive(SplitFields)]
pub struct SpherePos {
    /// Horizontal angle.
    pub polar: Angle<Coord>,
    /// Vertical angle.
    pub azimuth: Angle<Coord>,
}
