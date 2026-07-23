use super::*;

#[derive(SplitFields, Debug, Clone)]
pub struct FloatingText {
    pub text: Rc<str>,
    pub position: vec3<Coord>,
    pub velocity: vec3<Coord>,
    pub size: Coord,
    pub color: Color,
    pub lifetime: Bounded<Time>,
}

#[derive(SplitFields, Debug, Clone)]
pub struct Particle {
    pub position: vec3<Coord>,
    pub velocity: vec3<Coord>,
    pub radius: Coord,
    pub color: Color,
    pub lifetime: Bounded<Time>,
}

#[derive(SplitFields, Debug, Clone)]
pub struct SpawnParticles {
    pub density: R32,
    pub distribution: ParticleDistribution,
    pub z: Coord,
    pub size: RangeInclusive<Coord>,
    pub velocity: vec2<Coord>,
    pub lifetime: RangeInclusive<Time>,
    pub color: Color,
}

#[derive(Debug, Clone)]
pub enum ParticleDistribution {
    Circle { center: vec2<Coord>, radius: Coord },
    // Aabb(Aabb2<Coord>),
}

impl ParticleDistribution {
    pub fn sample(&self, rng: &mut impl Rng, density: R32) -> Vec<vec2<Coord>> {
        match *self {
            // ParticleDistribution::Aabb(aabb) => {
            //     let amount = density * aabb.width() * aabb.height();
            //     let extra = if rng.gen_bool(amount.fract().as_f32().into()) {
            //         1
            //     } else {
            //         0
            //     };
            //     let amount = (amount.floor()).as_f32() as usize + extra;

            //     (0..amount)
            //         .map(|_| {
            //             vec2(
            //                 rng.gen_range(aabb.min.x..=aabb.max.x),
            //                 rng.gen_range(aabb.min.y..=aabb.max.y),
            //             )
            //         })
            //         .collect()
            // }
            ParticleDistribution::Circle { center, radius } => {
                let amount = density * radius.sqr() * R32::PI;
                let extra = if rng.gen_bool(amount.fract().as_f32().into()) {
                    1
                } else {
                    0
                };
                let amount = (amount.floor()).as_f32() as usize + extra;

                (0..amount)
                    .map(|_| rng.gen_circle(center, radius))
                    .collect()
            }
        }
    }
}

impl Default for SpawnParticles {
    fn default() -> Self {
        Self {
            density: r32(5.0),
            distribution: ParticleDistribution::Circle {
                center: vec2::ZERO,
                radius: r32(0.5),
            },
            z: Coord::ZERO,
            size: r32(0.05)..=r32(0.15),
            velocity: vec2::ZERO,
            lifetime: r32(0.5)..=r32(1.5),
            color: Color::WHITE,
        }
    }
}
