mod logic;

use crate::prelude::*;

pub type Arena<V> =
    stecs::storage::slotmap::SlotMap<stecs::storage::slotmap::slotmap::DefaultKey, V>;

pub type Time = R32;
pub type Coord = R32;
pub type Science = i64;

pub struct Model {
    pub real_time: Time,
    pub camera: Camera2d,

    pub science: Science,
    pub planet: Planet,
}

impl Model {
    pub fn new() -> Self {
        let mut model = Self {
            real_time: Time::ZERO,
            camera: Camera2d {
                center: vec2::ZERO,
                rotation: Angle::ZERO,
                fov: Camera2dFov::Cover {
                    width: 50.0,
                    height: 50.0,
                    scale: 1.0,
                },
            },

            science: 0,
            planet: Planet::new(),
        };
        model.init();
        model
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
            orbit: Orbit::new(r32(13.0)),
        }
    }
}

pub const ORBIT_OBJECT_TRAIL_LEN: usize = 60;

pub struct Orbit {
    pub distance: Coord,
    pub satellites: StructOf<Arena<Satellite>>,
    pub debris: StructOf<Arena<Debris>>,
}

impl Orbit {
    pub fn new(distance: Coord) -> Self {
        Self {
            distance,
            satellites: default(),
            debris: default(),
        }
    }
}

#[derive(SplitFields, Debug, Clone)]
pub struct Satellite {
    pub position: SpherePos,
    pub velocity: SphereVelocity,
    pub radius: Coord,
    pub trail: VecDeque<SpherePos>,
    pub science_timer: Bounded<Time>,
}

#[derive(SplitFields, Debug, Clone, PartialEq, Eq)]
pub struct Debris {
    pub position: SpherePos,
    pub velocity: SphereVelocity,
    pub radius: Coord,
    pub trail: VecDeque<SpherePos>,
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

#[derive(SplitFields, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpherePos {
    pub distance: Coord,
    /// Horizontal angle.
    pub polar: Angle<Coord>,
    /// Vertical angle.
    pub azimuth: Angle<Coord>,
}

impl SpherePos {
    pub fn to_cartesian(self, anchor: vec2<Coord>) -> vec3<Coord> {
        let (polar_sin, polar_cos) = self.polar.sin_cos();
        let (azimuth_sin, azimuth_cos) = self.azimuth.sin_cos();
        anchor.extend(Coord::ZERO)
            + vec3(polar_sin * azimuth_cos, polar_sin * azimuth_sin, polar_cos) * self.distance
    }

    pub fn add_velocity(&mut self, velocity: SphereVelocity, delta_time: Time) {
        self.azimuth = (self.azimuth + velocity.azimuth * delta_time).normalized_2pi();
        self.polar = (self.polar + velocity.polar * delta_time).normalized_2pi();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SphereVelocity {
    /// Horizontal angle.
    pub polar: Angle<Coord>,
    /// Vertical angle.
    pub azimuth: Angle<Coord>,
}
