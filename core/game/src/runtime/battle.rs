use alloc::{collections::BTreeMap, vec::Vec};
use serde::{Deserialize, Serialize};

use crate::{Context, Error, Signal};

#[derive(Default, Serialize, Deserialize)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct PveBattleRuntime {
    pub enemies_action_flag: BTreeMap<u16, bool>,
    pub active_effects: Vec<u16>,
}

impl PveBattleRuntime {
    pub fn run(&mut self, _signal: &Signal, _ctx: &mut Context) -> Result<(), Error> {
        Ok(())
    }
}
