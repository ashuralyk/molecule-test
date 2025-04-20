use ckb_cinnabar_verifier::re_exports::ckb_std::ckb_types::packed::Script;
use common::hardcoded::XUDT_CODE_HASH;
use serde::{Deserialize, Serialize};

use crate::error::ScriptError;

#[repr(u8)]
pub enum ScriptType {
    GameData,
    TokenIssuer,
    PveSession,
    PvpSession,
}

impl TryFrom<u8> for ScriptType {
    type Error = ScriptError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::GameData),
            1 => Ok(Self::TokenIssuer),
            2 => Ok(Self::PveSession),
            3 => Ok(Self::PvpSession),
            _ => Err(ScriptError::UnknownScriptType),
        }
    }
}

impl From<ScriptType> for u8 {
    fn from(value: ScriptType) -> u8 {
        value as u8
    }
}

#[derive(Default, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct GameGlobal {
    pub action_point: u16,
    // pve
    pub pve_count: u16,
    pub pve_hunted_gold: u32,
    pub pve_easy_mode_count: u16,
    pub pve_killed_enemy_count: u16,
    pub pve_normal_mode_count: u16,
    pub pve_hard_mode_count: u16,
    pub pve_casued_damage: u32,
    pub pve_sufferred_damage: u32,
    pub pve_blocked_damage: u32,
    pub pve_healed_hp: u32,
    // pvp
    pub pvp_win_count: u16,
    pub pvp_lose_count: u16,
    pub pvp_looted_gold: u128,
    pub pvp_stolen_gold: u128,
}

pub struct GameConfig {}

impl Default for GameConfig {
    fn default() -> Self {
        Self {}
    }
}

impl GameConfig {
    pub fn is_xudt(&self, script: &Script) -> bool {
        script.code_hash().raw_data().as_ref() == XUDT_CODE_HASH
    }
}

#[derive(Serialize, Deserialize)]
pub struct PveSession {
    pub version: u8,
    pub action_point: u16,
    pub player_level: u8,
    pub material_hash: [u8; 32],
}
