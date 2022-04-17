use std::collections::HashMap;

pub use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::Coordinate;

#[derive(Deserialize)]
pub struct GameState {
    #[serde(default)]
    pub member: bool,
    #[serde(default)]
    pub skill_levels: HashMap<String, u8>,
    #[serde(default)]
    pub items: HashMap<String, u32>,
    #[serde(default)]
    pub varps: HashMap<u32, i32>,
    #[serde(default)]
    pub varbits: HashMap<u32, i32>,
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
pub enum Compare {
    LT, LE, EQ, GE, GT, NOT
}

impl Compare {
    pub fn test<T: Ord>(&self, a: T, b: T) -> bool {
        match self {
            Compare::LT => a < b,
            Compare::LE => a <= b,
            Compare::EQ => a == b,
            Compare::GE => a >= b,
            Compare::GT => a > b,
            Compare::NOT => a != b,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum RequirementDefinition {
    Membership,
    Skill { skill: String, level: u8 },
    Item { #[serde(with = "serde_regex")] item: Regex, quantity: u32 },
    Varp { index: u32, value: i32, compare: Compare },
    Varbit { index: u32, value: i32, compare: Compare },
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
            RequirementDefinition::Varp { index, value, compare } => game_state.varps.get(index).map(|val| compare.test(value, val)).unwrap_or(false),
            RequirementDefinition::Varbit { index, value, compare } => game_state.varbits.get(index).map(|val| compare.test(value, val)).unwrap_or(false),
        }
    }
}
