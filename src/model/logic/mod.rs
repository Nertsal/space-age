mod action;
mod particles;
mod update;

use super::*;

use crate::util::random_angle;

impl Model {
    pub fn init(&mut self) {}

    pub fn get_stat(&self, stat: Stat) -> R32 {
        self.stats.get(&stat).copied().unwrap_or(R32::ONE)
    }

    pub fn get_research_state(&self, id: u64) -> ResearchState {
        if self.researched.contains(&id) {
            return ResearchState::Researched;
        }

        let Some(research) = self.config.research.items.iter().find(|item| item.id == id) else {
            return ResearchState::Locked;
        };
        if research.after.iter().all(|id| self.researched.contains(id)) {
            ResearchState::Available {
                cost: research.cost,
            }
        } else {
            ResearchState::Locked
        }
    }

    pub fn collision_risk(&self) -> CollisionRisk {
        let orbit = &self.planet.orbit;
        let satellites = orbit.satellites.ids().count();
        let debris = orbit.debris.ids().count();
        let total = satellites + debris;
        if total <= 5 {
            CollisionRisk::Safe
        } else if total <= 8 {
            CollisionRisk::Caution
        } else if total <= 15 {
            CollisionRisk::Moderate
        } else {
            CollisionRisk::Severe
        }
    }

    pub fn active_satellites(&self) -> usize {
        let orbit = &self.planet.orbit;
        query!(orbit.satellites, (&lifetime))
            .filter(|lifetime| lifetime.is_above_min())
            .count()
    }

    pub fn inactive_satellites(&self) -> usize {
        let orbit = &self.planet.orbit;
        query!(orbit.satellites, (&lifetime))
            .filter(|lifetime| lifetime.is_min())
            .count()
    }

    pub fn debris(&self) -> usize {
        let orbit = &self.planet.orbit;
        orbit.debris.ids().count()
    }
}
