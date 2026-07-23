use super::*;

use crate::model::*;

#[derive(geng::asset::Load, Serialize, Deserialize, Debug, Clone)]
#[load(serde = "ron")]
pub struct Config {
    pub home_planet: PlanetConfig,
    pub theoretic_research: Science,
    pub satellites: HashMap<SatelliteKind, SatelliteConfig>,

    pub research: ResearchConfig,
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

impl Default for SatelliteConfig {
    fn default() -> Self {
        Self {
            launch_cost: 0,
            science: 0,
            interval: Time::ONE,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResearchConfig {
    pub items: Vec<ResearchItemConfig>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResearchItemConfig {
    pub name: Arc<str>,
    pub description: Arc<str>,
    pub id: u64,
    pub after: Vec<u64>,
    pub cost: Science,
    pub effect: Research,
}
