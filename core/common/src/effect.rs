use serde::{Deserialize, Serialize};

use crate::{enum_with_display, value::ValueType};

#[derive(Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct EffectConfig {
    pub name: EffectName,
    pub trap: bool,
    pub owner_source: Option<bool>,
    pub owner_target: Option<bool>,
    pub value: Option<ValueType>,
    pub countdown: Option<ValueType>,
}

enum_with_display!(
    #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, PartialOrd)]
    #[cfg_attr(feature = "debug", derive(Debug))]
    pub enum EffectName {
        ExtraDamage,
    }
);

#[derive(Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Effect {
    pub name: EffectName,
    pub trap: bool,
    pub owner_source: Option<bool>,
    pub owner_target: Option<bool>,
    pub value: Option<u8>,
    pub countdown: Option<u8>,
}

pub fn roulette_effect(effect: EffectConfig, seed: u64) -> Effect {
    let seeds = seed.to_le_bytes();
    Effect {
        name: effect.name,
        trap: effect.trap,
        owner_source: effect.owner_source,
        owner_target: effect.owner_target,
        value: effect.value.map(|param| param.value_u8(seeds[0])),
        countdown: effect.countdown.map(|param| param.value_u8(seeds[1])),
    }
}
