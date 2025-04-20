use alloc::vec::Vec;
use serde::{Deserialize, Serialize};
use serde_molecule::dynvec_serde;

use crate::value::ValueType;

#[derive(Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct PlayerConfig {
    pub level: u8,
    pub hp: ValueType,
    pub energy: u8,
    pub attack: ValueType,
    pub defense: ValueType,
    pub spirit: ValueType,
    pub initial_handhold_capacity: u8,
    pub initial_deck_capacity: u8,
    pub initial_equipment_capacity: u8,
    pub initial_sorcery_capacity: u8,
    pub max_handhold_capacity: u8,
    pub max_equipment_capacity: u8,
    pub max_sorcery_capacity: u8,
    pub heal_action_point: u8,
    pub discard_action_point: u8,
    pub easy_action_point: u8,
    pub normal_action_point: u8,
    pub hard_action_point: u8,
}

#[derive(Serialize, Deserialize)]
#[serde(transparent)]
pub struct PlayerPool {
    #[serde(with = "dynvec_serde")]
    pub inner: Vec<PlayerConfig>,
}

#[derive(Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Player {
    pub level: u8,
    pub hp: u16,
    pub energy: u8,
    pub attack: u8,
    pub defense: u8,
    pub spirit: u8,
    pub initial_handhold_capacity: u8,
    pub initial_deck_capacity: u8,
    pub initial_equipment_capacity: u8,
    pub initial_sorcery_capacity: u8,
    pub max_handhold_capacity: u8,
    pub max_equipment_capacity: u8,
    pub max_sorcery_capacity: u8,
    pub heal_action_point: u8,
    pub discard_action_point: u8,
    pub easy_action_point: u8,
    pub normal_action_point: u8,
    pub hard_action_point: u8,
}

pub fn roulette_player(player_pool_bin: &[u8], player_level: u8, seed: u64) -> Option<Player> {
    let player_pool: PlayerPool = serde_molecule::from_slice(player_pool_bin, false).ok()?;
    let template = player_pool
        .inner
        .into_iter()
        .find(|player| player.level == player_level)?;
    let seeds = seed.to_le_bytes();
    Some(Player {
        level: template.level,
        hp: template.hp.value_u16(seeds[0]),
        energy: template.energy,
        attack: template.attack.value_u8(seeds[1]),
        defense: template.defense.value_u8(seeds[2]),
        spirit: template.spirit.value_u8(seeds[3]),
        initial_handhold_capacity: template.initial_handhold_capacity,
        initial_deck_capacity: template.initial_deck_capacity,
        initial_equipment_capacity: template.initial_equipment_capacity,
        initial_sorcery_capacity: template.initial_sorcery_capacity,
        max_handhold_capacity: template.max_handhold_capacity,
        max_equipment_capacity: template.max_equipment_capacity,
        max_sorcery_capacity: template.max_sorcery_capacity,
        heal_action_point: template.heal_action_point,
        discard_action_point: template.discard_action_point,
        easy_action_point: template.easy_action_point,
        normal_action_point: template.normal_action_point,
        hard_action_point: template.hard_action_point,
    })
}
