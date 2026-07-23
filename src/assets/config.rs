use super::*;

use crate::model::{Coord, Science, Time};

#[derive(geng::asset::Load, Serialize, Deserialize, Debug, Clone)]
#[load(serde = "ron")]
pub struct Config {
    pub home_planet: PlanetConfig,
    pub theoretic_research: Science,
    pub satellite: SatelliteConfig,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlanetConfig {
    pub radius: Coord,
    pub orbit_distance: Coord,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SatelliteConfig {
    pub launch_cost: Science,
    pub science: Science,
    pub interval: Time,
}
