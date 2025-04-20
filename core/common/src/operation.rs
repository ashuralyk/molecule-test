use alloc::vec::Vec;
use serde::{Deserialize, Serialize};
use serde_molecule::dynvec_serde;

use crate::enemy::EnemyLevel;

#[derive(Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub enum GameOperation {
    StartGame,
    StartBattle(EnemyLevel),
    RoundOver,
    SpellCard(u16, Option<u16>),
    SelectCard(Vec<u16>),
    HealHp,
    DestroyCard,
}

#[derive(Serialize, Deserialize)]
#[serde(transparent)]
pub struct GameOperationSet {
    #[serde(with = "dynvec_serde")]
    pub operations: Vec<GameOperation>,
}

impl GameOperationSet {
    pub fn new(operations: Vec<GameOperation>) -> Self {
        Self { operations }
    }

    pub fn to_vec(operations: Vec<GameOperation>) -> Vec<u8> {
        let operations_set = Self::new(operations);
        serde_molecule::to_vec(&operations_set, false).unwrap()
    }
}
