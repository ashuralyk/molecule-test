use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use serde::{Deserialize, Serialize};
use serde_molecule::dynvec_serde;

use crate::{enum_with_display, value::ValueType};

enum_with_display!(
    #[derive(Serialize, Deserialize, Clone, Copy, PartialEq)]
    #[cfg_attr(feature = "debug", derive(Debug))]
    pub enum CardCategory {
        Attack,
        Defense,
        Spirit,
        Recover,
        Skill,
        Trap,
        Sorcery,
        Equipment,
    }
);

enum_with_display!(
    #[derive(Serialize, Deserialize, Clone, Copy, PartialEq)]
    #[cfg_attr(feature = "debug", derive(Debug))]
    pub enum CardTarget {
        Player,
        Enemy,
        Entity,
        RandomEnemy,
        AllEnemies,
        AllEntities,
    }
);

enum_with_display!(
    #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, PartialOrd)]
    #[cfg_attr(feature = "debug", derive(Debug))]
    pub enum CardName {
        Attack,
    }
);

#[derive(Serialize, Deserialize, Clone, Copy)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub enum AwakeTrigger {
    Cost,
    Damage,
    Hurt,
    Round,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AwakeConfig {
    pub awake_type: AwakeTrigger,
    pub value: ValueType,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CardConfig {
    pub version: u8,
    pub name: CardName,
    pub rarity: u8,
    pub exile: bool,
    pub cost: ValueType,
    pub awake: Option<AwakeConfig>,
    pub category: CardCategory,
    pub target: CardTarget,
    pub description: String,
    pub value_0: Option<ValueType>,
    pub value_1: Option<ValueType>,
}

#[derive(Serialize, Deserialize)]
#[serde(transparent)]
pub struct CardPool {
    #[serde(with = "dynvec_serde")]
    pub inner: Vec<CardConfig>,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Awake {
    pub awake_type: AwakeTrigger,
    pub value: u8,
}

#[derive(Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Card {
    pub seed: u64,
    pub name: CardName,
    pub golden: bool,
    pub target: CardTarget,
    pub cost: u8,
    pub exile: bool,
    pub awake: Option<Awake>,
    pub category: CardCategory,
    pub description: String,
    pub value_0: u8,
    pub value_1: u8,
}

impl Card {
    pub fn descripted(&mut self) {
        self.description = {
            let mut parts = self
                .description
                .split("{}")
                .map(ToString::to_string)
                .collect::<Vec<_>>();
            if parts.is_empty() {
                return;
            }
            [self.value_0, self.value_1]
                .into_iter()
                .filter(|v| v > &0)
                .enumerate()
                .for_each(|(i, v)| {
                    parts.insert(2 * i + 1, v.to_string());
                });
            parts.join("")
        };
    }
}

pub fn instance_card_by_seed(card: &CardConfig, seed: u64) -> Card {
    let seeds = seed.to_le_bytes();
    let mut card = Card {
        seed,
        name: card.name,
        golden: false,
        target: card.target,
        cost: card.cost.value_u8(seeds[0]),
        exile: card.exile,
        awake: card.awake.clone().map(|awake| Awake {
            awake_type: awake.awake_type,
            value: awake.value.value_u8(seeds[1]),
        }),
        category: card.category,
        description: card.description.clone(),
        value_0: card
            .value_0
            .as_ref()
            .map(|param| param.value_u8(seeds[2]))
            .unwrap_or_default(),
        value_1: card
            .value_1
            .as_ref()
            .map(|param| param.value_u8(seeds[3]))
            .unwrap_or_default(),
    };
    card.descripted();
    card
}

pub fn get_card_template_by_name(card_pool_bin: &[u8], name: CardName) -> Option<CardConfig> {
    let card_pool: CardPool =
        serde_molecule::from_slice(card_pool_bin, false).expect("card pool bin broken");
    card_pool.inner.into_iter().find(|card| card.name == name)
}
