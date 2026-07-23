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
}
