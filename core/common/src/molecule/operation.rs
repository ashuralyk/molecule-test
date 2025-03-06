use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum EnemyLevel {
    Easy,
    Normal,
    Hard,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Operation {
    pub flag: OperationType,
    pub payload: Vec<u8>,
}

#[derive(Serialize, Deserialize, Clone)]
#[repr(u8)]
pub enum OperationType {
    StartGame,
    StartBattle,
    RoundOver,
    SpellCard,
    SelectCard,
    HealHp,
    DestroyCard,
}

impl From<u8> for OperationType {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::StartGame,
            1 => Self::StartBattle,
            2 => Self::RoundOver,
            3 => Self::SpellCard,
            4 => Self::SelectCard,
            5 => Self::HealHp,
            6 => Self::DestroyCard,
            _ => panic!("Invalid operation type"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StartBattleParameters {
    pub enemy_level: EnemyLevel,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SpellCardParameters {
    pub card_runtime_id: u16,
    pub target_runtime_id: Option<u16>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SelectCardParameters {
    pub card_runtime_id_set: Vec<u16>,
}
