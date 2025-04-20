use alloc::vec::Vec;
use common::enemy::{Action, Enemy, WeightedAction};
use serde::{Deserialize, Serialize};
use serde_molecule::dynvec_serde;

use crate::{Context, Error, Signal};

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct EnemyRuntime {
    pub raw: Enemy,
    pub runtime_id: u16,
    pub hp: u16,
    pub attack: u8,
    pub defense: u8,
    pub spirit: u8,
    pub block: u16,
    pub shield: u16,
    #[serde(with = "dynvec_serde")]
    pub action_pool: Vec<WeightedAction>,
    #[serde(with = "dynvec_serde")]
    pub active_actions: Vec<Action>,
    pub active_effects: Vec<u16>,
}

impl EnemyRuntime {
    pub fn run(&mut self, _signal: &Signal, _ctx: &mut Context) -> Result<(), Error> {
        Ok(())
    }
}
