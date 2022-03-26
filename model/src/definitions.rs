use std::collections::HashMap;

pub use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::Coordinate;

#[derive(Deserialize)]
pub struct GameState {
    pub member: bool,
    pub skill_levels: HashMap<String, u8>,
    pub items: HashMap<String, u32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum EdgeDefinition {
    Door { id: u32, position: Coordinate, #[serde(with = "serde_regex")] action: Regex },
    GameObject { id: u32, position: Coordinate, #[serde(with = "serde_regex")] action: Regex },
    SpellTeleport { spell: String },
    ItemTeleport { #[serde(with = "serde_regex")] item: Regex, #[serde(with = "serde_regex")] action: Regex },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum RequirementDefinition {
    Membership,
    Skill { skill: String, level: u8 },
    Item { #[serde(with = "serde_regex")] item: Regex, quantity: u32 },
}

impl RequirementDefinition {
    pub fn is_met(&self, game_state: &GameState) -> bool {
        match self {
            RequirementDefinition::Membership => game_state.member,
            RequirementDefinition::Skill { skill, level } => game_state.skill_levels.get(skill).unwrap_or(&1) >= level,
            RequirementDefinition::Item { item, quantity } => {
                let total: u32 = game_state.items.iter()
                    .filter(|(i, _)| item.is_match(i))
                    .map(|(_, q)| q)
                    .sum();
                total >= *quantity
            },
        }
    }
}
