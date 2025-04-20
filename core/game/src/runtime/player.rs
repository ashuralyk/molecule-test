use alloc::vec::Vec;
use common::player::Player;
use serde::{Deserialize, Serialize};

use crate::{Context, Error, Signal};

#[cfg(feature = "log")]
use crate::log::CardMovement;

// Runtime state for a player that tracks attributes, card collections,
// and effects during gameplay. Manages core game mechanics like deck/hand
// management and player stats.
#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct PlayerRuntime {
    pub raw: Player,
    pub runtime_id: u16,
    pub energy: u8,
    pub hp: u16,
    pub attack: u8,
    pub defense: u8,
    pub spirit: u8,
    pub block: u16,
    pub shield: u16,
    pub equipment_cards: Vec<u16>,
    pub sorcery_cards: Vec<u16>,
    pub handhold_cards: Vec<u16>,
    pub deck_cards: Vec<u16>,
    pub grave_cards: Vec<u16>,
    pub exile_cards: Vec<u16>,
    pub active_effects: Vec<u16>,
}

impl PlayerRuntime {
    pub fn run(&mut self, _signal: &Signal, _ctx: &mut Context) -> Result<(), Error> {
        Ok(())
    }
}
