use alloc::{vec, vec::Vec};

use ckb_cinnabar_verifier::re_exports::ckb_std::ckb_types::{packed::Script, prelude::Unpack};
use common::{
    contract::SporeData,
    hardcoded::{CKB_DECIMAL, SPORE_CODE_HASH_SET, TYPE_BURN_CODE_HASH, XUDT_CODE_HASH},
};
use game_core::GameStatistics;
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

impl GameGlobal {
    pub fn action_point_charge_mode(&self, next: &Self) -> bool {
        self.action_point < next.action_point
            && self.pvp_win_count == next.pvp_win_count
            && self.pvp_lose_count == next.pvp_lose_count
            && self.pve_count == next.pve_count
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

    pub fn statistics_equal(&self, other: &Self, stats: &GameStatistics) -> bool {
        other.pve_easy_mode_count == self.pve_easy_mode_count + stats.easy_mode_count
            || other.pve_normal_mode_count == self.pve_normal_mode_count + stats.normal_mode_count
            || other.pve_hard_mode_count == self.pve_hard_mode_count + stats.hard_mode_count
            || other.pve_casued_damage == self.pve_casued_damage + stats.casued_damage as u32
            || other.pve_blocked_damage == self.pve_blocked_damage + stats.blocked_damage as u32
            || other.pve_healed_hp == self.pve_healed_hp + stats.healed_hp as u32
            || other.pve_sufferred_damage
                == self.pve_sufferred_damage + stats.sufferred_damage as u32
            || other.pve_killed_enemy_count
                == self.pve_killed_enemy_count + stats.killed_enemy_count
    }
}

pub struct GameConfig {
    pub protocol_payee_scripts: Vec<Vec<u8>>,
    pub block_per_action_point: u8,
    pub ckb_per_action_point: u64,
    pub player_level_ckb_costs: Vec<u64>,
    pub dob_card_clusters: Vec<[u8; 32]>,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            protocol_payee_scripts: vec![
                // Native Test (always_success_script(vec![0]))
                vec![
                    54, 0, 0, 0, 16, 0, 0, 0, 48, 0, 0, 0, 49, 0, 0, 0, 230, 131, 176, 65, 57, 52,
                    71, 104, 52, 132, 153, 194, 62, 177, 50, 109, 90, 82, 214, 219, 0, 108, 13, 47,
                    236, 224, 10, 131, 31, 54, 96, 215, 2, 1, 0, 0, 0, 0,
                ],
                // Testnet
                vec![
                    73, 0, 0, 0, 16, 0, 0, 0, 48, 0, 0, 0, 49, 0, 0, 0, 155, 215, 224, 111, 62,
                    207, 75, 224, 242, 252, 210, 24, 139, 35, 241, 185, 252, 200, 142, 93, 75, 101,
                    168, 99, 123, 23, 114, 59, 189, 163, 204, 232, 1, 20, 0, 0, 0, 148, 15, 4, 172,
                    228, 186, 159, 46, 89, 239, 26, 26, 246, 249, 157, 158, 234, 37, 202, 141,
                ],
            ],
            block_per_action_point: 20,
            ckb_per_action_point: 10 * CKB_DECIMAL,
            player_level_ckb_costs: vec![0, 0, 500 * CKB_DECIMAL, 1000 * CKB_DECIMAL],
            dob_card_clusters: vec![
                // Native Test
                [
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                ],
                // Testnet
                [
                    0x53, 0x26, 0x97, 0xf3, 0x9c, 0x3c, 0xa0, 0x3b, 0xd3, 0x3b, 0x30, 0x93, 0x2b,
                    0xa5, 0xf4, 0xbb, 0x11, 0x31, 0x6d, 0x12, 0x0a, 0xf6, 0x86, 0xcd, 0x87, 0x02,
                    0x03, 0x44, 0x55, 0x1d, 0xce, 0x43,
                ],
            ],
        }
    }
}

impl GameConfig {
    pub fn is_type_burn(&self, script: &Script) -> bool {
        TYPE_BURN_CODE_HASH.contains(&script.code_hash().unpack())
    }

    pub fn is_spore(&self, script: &Script) -> bool {
        SPORE_CODE_HASH_SET.contains(&script.code_hash().unpack())
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

    pub fn player_level_cost(&self, level: u8) -> Result<u64, ScriptError> {
        self.player_level_ckb_costs
            .get(level as usize)
            .ok_or(ScriptError::PlayerLevelOutOfRange)
            .cloned()
    }
}

#[derive(Serialize, Deserialize)]
pub struct PveSession {
    pub version: u8,
    pub action_point: u16,
    pub player_level: u8,
    pub material_hash: [u8; 32],
}
