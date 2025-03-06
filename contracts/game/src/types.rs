use alloc::{vec, vec::Vec};

use ckb_cinnabar_verifier::re_exports::ckb_std::ckb_types::packed::Script;
use common::{
    hardcoded::{SPORE_CODE_HASH_SET, XUDT_CODE_HASH},
    molecule::{Operation, SporeData},
};
use serde::{Deserialize, Serialize};
use serde_molecule::dynvec_serde;

use crate::error::ScriptError;

#[repr(u8)]
pub enum ScriptType {
    GameData,
    TokenIssuer,
    PveSession,
    LockedCard,
    PvpSession,
}

impl TryFrom<u8> for ScriptType {
    type Error = ScriptError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::GameData),
            1 => Ok(Self::TokenIssuer),
            2 => Ok(Self::PveSession),
            3 => Ok(Self::LockedCard),
            4 => Ok(Self::PvpSession),
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

impl GameGlobal {
    pub fn spore_lock_mode(&self, next: &Self) -> bool {
        self.pve_count == next.pve_count
            && self.pvp_win_count == next.pvp_win_count
            && self.pvp_lose_count == next.pvp_lose_count
            && self.action_point < next.action_point
    }

    pub fn pve_session_create_mode(&self, next: &Self) -> bool {
        self.pve_count == next.pve_count
            && self.pvp_win_count == next.pvp_win_count
            && self.pvp_lose_count == next.pvp_lose_count
            && next.action_point == 0
    }

    pub fn pve_session_settlement_mode(&self, next: &Self) -> bool {
        self.pve_count + 1 == next.pve_count
    }

    pub fn pvp_session_settlement_mode(&self, next: &Self) -> bool {
        self.pvp_win_count + 1 == next.pvp_win_count
            || self.pvp_lose_count + 1 == next.pvp_lose_count
    }

    pub fn pve_equal(&self, other: &Self) -> bool {
        self.pve_count == other.pve_count
            && self.pve_hunted_gold == other.pve_hunted_gold
            && self.pve_killed_enemy_count == other.pve_killed_enemy_count
            && self.pve_easy_mode_count == other.pve_easy_mode_count
            && self.pve_normal_mode_count == other.pve_normal_mode_count
            && self.pve_hard_mode_count == other.pve_hard_mode_count
            && self.pve_casued_damage == other.pve_casued_damage
            && self.pve_sufferred_damage == other.pve_sufferred_damage
            && self.pve_blocked_damage == other.pve_blocked_damage
            && self.pve_healed_hp == other.pve_healed_hp
    }

    pub fn pvp_equal(&self, other: &Self) -> bool {
        self.pvp_win_count == other.pvp_win_count
            && self.pvp_lose_count == other.pvp_lose_count
            && self.pvp_looted_gold == other.pvp_looted_gold
            && self.pvp_stolen_gold == other.pvp_stolen_gold
    }
}

pub struct GameConfig {
    pub action_point_per_block: u8,
    pub card_redeemable_blocks: u64,
    pub dob_card_clusters: Vec<[u8; 32]>,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            action_point_per_block: 1,
            card_redeemable_blocks: 100,
            dob_card_clusters: vec![
                // Native Test
                [
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                ],
                // Testnet
                [
                    0x16, 0x03, 0x20, 0x59, 0xd3, 0x8f, 0x19, 0x23, 0x6c, 0xa5, 0x69, 0x35, 0x55,
                    0xf8, 0xc9, 0xe1, 0x74, 0x7a, 0xec, 0x60, 0xc4, 0xdb, 0x8a, 0xf1, 0xbf, 0x8a,
                    0x5e, 0xb7, 0x44, 0x20, 0x3a, 0x8f,
                ],
            ],
        }
    }
}

impl GameConfig {
    pub fn is_spore(&self, script: &Script) -> bool {
        let code_hash = script
            .code_hash()
            .raw_data()
            .to_vec()
            .try_into()
            .unwrap_or_default();
        SPORE_CODE_HASH_SET.contains(&code_hash)
    }

    pub fn is_xudt(&self, script: &Script) -> bool {
        script.code_hash().raw_data().as_ref() == XUDT_CODE_HASH
    }

    pub fn is_valid_cluster(&self, spore_data: &SporeData) -> bool {
        spore_data
            .cluster_id
            .as_ref()
            .map(|cluster_id| {
                self.dob_card_clusters
                    .iter()
                    .any(|id| id[..] == cluster_id[..])
            })
            .unwrap_or(false)
    }
}

#[derive(Serialize, Deserialize)]
pub struct PveSession {
    pub action_point: u16,
    pub player_level: u8,
    pub collection_hash: [u8; 32],
}

#[derive(Serialize, Deserialize)]
#[serde(transparent)]
pub struct PveWitness {
    #[serde(with = "dynvec_serde")]
    pub operations: Vec<Operation>,
}
