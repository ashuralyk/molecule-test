use alloc::vec::Vec;
use serde::{Deserialize, Serialize};
use serde_molecule::dynvec_serde;

use crate::{
    effect::{Effect, EffectConfig},
    enum_with_display,
    value::ValueType,
};

#[derive(Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub enum ActionConfig {
    #[serde(rename = "attack")]
    Attack(ValueType),
    #[serde(rename = "multiple_attack")]
    MultipleAttack(ValueType, ValueType),
    #[serde(rename = "defense")]
    Defense(ValueType),
    #[serde(rename = "shield")]
    Shield(ValueType),
    #[serde(rename = "effect")]
    Effect(EffectConfig),
    #[serde(rename = "summon_creature")]
    SummonCreature(EnemyLevel, u8),
    #[serde(rename = "add_attack")]
    AddAttack(ValueType),
    #[serde(rename = "add_defense")]
    AddDefense(ValueType),
    #[serde(rename = "add_shield")]
    AddShield(ValueType),
    #[serde(rename = "add_effect")]
    AddEffect(EffectConfig),
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub enum EnemyLevel {
    Easy,
    Normal,
    Hard,
}

impl From<u8> for EnemyLevel {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Easy,
            1 => Self::Normal,
            2 => Self::Hard,
            _ => panic!("Invalid enemy level"),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct WeightedActionConfig {
    pub action: ActionConfig,
    pub weight: u16,
    pub amount: Option<u8>,
    #[serde(with = "dynvec_serde")]
    pub tweakers: Vec<WeightTweaker>,
}

#[repr(usize)]
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub enum Condition {
    #[serde(rename = "with_born")]
    WithBorn,
    #[serde(rename = "hp_down")]
    HpDown,
    #[serde(rename = "hp_percent_70")]
    HpPercent70,
    #[serde(rename = "hp_percent_50")]
    HpPercent50,
    #[serde(rename = "hp_percent_30")]
    HpPercent30,
    #[serde(rename = "shield_broken")]
    ShieldBroken,
    #[serde(rename = "effected")]
    Effected,
    #[serde(rename = "partner_dead")]
    PartnerDead,
    #[serde(skip)]
    _LENGTH_,
}

#[derive(Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct WeightTweaker {
    pub condition: Condition,
    pub threshold: u8,
    pub weight: u16,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct EnemyConfig {
    pub version: u8,
    pub name: EnemyName,
    pub level: EnemyLevel,
    pub hp: ValueType,
    pub gold: ValueType,
    pub attack: ValueType,
    pub defense: ValueType,
    pub spirit: ValueType,
    pub powerup_threshold: ValueType,
    #[serde(with = "dynvec_serde")]
    pub actions: Vec<WeightedActionConfig>,
}

enum_with_display!(
    #[derive(Serialize, Deserialize, Clone, Copy, Debug)]
    pub enum EnemyName {
        Goblin,
        Orc,
        Troll,
        Dragon,
        Demon,
        Angel,
        God,
        // ...
    }
);

#[derive(Serialize, Deserialize)]
#[serde(transparent)]
pub struct EnemyPool {
    #[serde(with = "dynvec_serde")]
    pub inner: Vec<EnemyConfig>,
}

#[derive(Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Enemy {
    pub seed: u64,
    pub name: EnemyName,
    pub level: EnemyLevel,
    pub hp: u16,
    pub gold: u16,
    pub attack: u8,
    pub defense: u8,
    pub spirit: u8,
    pub powerup_threshold: u8,
    #[serde(with = "dynvec_serde")]
    pub actions: Vec<WeightedActionConfig>,
}

#[derive(Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub enum Action {
    UseAttack(u8),
    UseMultipleAttack(u8, u8),
    UseDefense(u8),
    UseShield(u8),
    UseEffect(Effect),
    SummonCreature(EnemyLevel, u8),
    AddEnemyAttack(u8),
    AddEnemyDefense(u8),
    AddEnemeyShield(u8),
    AddEnemeyEffect(Effect),
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct WeightedAction {
    pub raw: WeightedActionConfig,
    pub weight: u16,
    pub use_count: u8,
    pub condition_hit: [u8; Condition::_LENGTH_ as usize],
}
