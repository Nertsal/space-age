mod logic;
mod particles;

pub use self::particles::*;

use crate::prelude::*;

pub type Arena<V> =
    stecs::storage::slotmap::SlotMap<stecs::storage::slotmap::slotmap::DefaultKey, V>;
pub type ArenaId = stecs::storage::slotmap::slotmap::DefaultKey;

pub type Time = R32;
pub type Coord = R32;
pub type Science = i64;

pub struct Model {
    pub config: Config,
    pub real_time: Time,
    pub camera: Camera2d,

    pub researched: HashSet<u64>,
    pub abilities: HashSet<Ability>,
    pub stats: HashMap<Stat, R32>,

    pub science: Science,
    pub planet: Planet,
    pub particles: StructOf<Arena<Particle>>,
    pub queued_particles: Vec<SpawnParticles>,

    pub theory_progress: R32,

    pub hovered_object: Option<InteractiveId>,
    pub selected_object: Option<InteractiveId>,
    pub hovered_rotation: Angle<Coord>,
    pub selected_rotation: Angle<Coord>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractiveId {
    Satellite(ArenaId),
    Debris(ArenaId),
}

impl Model {
    pub fn new(config: &Config) -> Self {
        let mut model = Self {
            config: config.clone(),
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

            researched: HashSet::new(),
            abilities: hashset! { Ability::TheoreticResearch },
            stats: HashMap::new(),

            science: 0,
            planet: Planet::new(&config.home_planet),
            particles: default(),
            queued_particles: Vec::new(),

            theory_progress: R32::ZERO,

            hovered_object: None,
            selected_object: None,
            hovered_rotation: Angle::ZERO,
            selected_rotation: Angle::ZERO,
        };
        model.init();
        model
    }
}

#[derive(Debug, Clone)]
pub enum ResearchState {
    Researched,
    Available { cost: Science },
    Locked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Research {
    Upgrade(Stat, R32),
    Unlock(Ability),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Stat {
    Theorycrafting,
    SatelliteLongevity,
    SatelliteEfficiency,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Ability {
    TheoreticResearch,
    Launch(SatelliteKind),
    Deorbit,
    DeorbitAuto,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    TheoreticResearch,
    Launch(SatelliteKind),
    Deorbit(InteractiveId),
}

impl Action {
    pub fn ability(&self) -> Ability {
        match self {
            Action::TheoreticResearch => Ability::TheoreticResearch,
            Action::Launch(kind) => Ability::Launch(*kind),
            Action::Deorbit(_) => Ability::Deorbit,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SatelliteKind {
    Basic,
    Communication,
}

pub struct Planet {
    /// Position of the planet in the solar system.
    pub position: PolarPos,
    pub radius: Coord,
    pub orbit: Orbit,
}

impl Planet {
    pub fn new(config: &PlanetConfig) -> Self {
        Self {
            position: PolarPos {
                distance: Coord::ZERO,
                angle: Angle::ZERO,
            },
            radius: config.radius,
            orbit: Orbit::new(config.radius + config.orbit_distance),
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
    pub kind: SatelliteKind,
    pub position: SpherePos,
    pub velocity: SphereVelocity,
    pub visual_radius: Coord,
    pub radius: Coord,
    pub trail: VecDeque<SpherePos>,
    pub science_timer: Bounded<Time>,
    pub lifetime: Bounded<Time>,
    pub deorbiting: bool,
}

#[derive(SplitFields, Debug, Clone, PartialEq, Eq)]
pub struct Debris {
    pub position: SpherePos,
    pub velocity: SphereVelocity,
    pub visual_radius: Coord,
    pub radius: Coord,
    pub trail: VecDeque<SpherePos>,
    pub deorbiting: bool,
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

    pub fn unit_vec(&self) -> vec3<Coord> {
        Self {
            distance: Coord::ONE,
            ..*self
        }
        .to_cartesian(vec2::ZERO)
    }

    pub fn add_velocity(&mut self, velocity: SphereVelocity, delta_time: Time) {
        let a = self.unit_vec();
        let k = velocity.axis;

        let (sin, cos) = (velocity.speed * delta_time).sin_cos();
        let b = a * cos + vec3::cross(k, a) * sin + k * vec3::dot(k, a) * (Coord::ONE - cos);

        self.polar = Angle::atan2((b.x.sqr() + b.y.sqr()).sqrt(), b.z);
        self.azimuth = Angle::atan2(b.y, b.x);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SphereVelocity {
    /// Angle change.
    pub speed: Angle<Coord>,
    /// Rotation axis.
    pub axis: vec3<Coord>,
}
