use alloc::{collections::BTreeMap, vec, vec::Vec};
use common::enemy::EnemyLevel;
use serde::{Deserialize, Serialize};

use crate::{
    runtime::{RuntimePool, COUNTERPARTY_RUNTIME_ID},
    Signal,
};

#[cfg(feature = "log")]
use crate::Log;

#[derive(Default, Serialize, Deserialize)]
pub struct GameStatistics {
    pub easy_mode_count: u16,
    pub normal_mode_count: u16,
    pub hard_mode_count: u16,
    pub killed_enemy_count: u16,
    pub casued_damage: u16,
    pub sufferred_damage: u16,
    pub blocked_damage: u16,
    pub healed_hp: u16,
}

#[derive(Default, Serialize, Deserialize)]
pub struct RandGenerator {
    state: u64,
}

impl RandGenerator {
    pub fn new(seed: u64) -> Self {
        let seed = if seed == 0 {
            0x1234_5678_9abc_def0 // default seed if 0 is provided
        } else {
            seed
        };
        Self { state: seed }
    }

    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.state = x;
        x.wrapping_mul(0x2545_f491_4f6c_dd1d)
    }
}

#[derive(Default, Serialize, Deserialize)]
pub struct Context {
    pub rng: RandGenerator,
    pub statistics: GameStatistics,
    pub runtimes: RuntimePool,
    runtime_id: u16,
    battle_count: u8,
    #[serde(skip)]
    #[serde(default)]
    current_battle_round: u8,
    #[serde(skip)]
    #[serde(default)]
    battle_level: Option<EnemyLevel>,
    #[serde(skip)]
    #[serde(default)]
    signals: Vec<Signal>,
    #[serde(skip)]
    #[serde(default)]
    signal_history: BTreeMap<u8, Vec<Signal>>,
    #[serde(skip)]
    #[serde(default)]
    delete_runtime_history: Vec<u16>,
    #[cfg(feature = "log")]
    #[serde(skip)]
    #[serde(default)]
    logs: Vec<Log>,
}

impl Context {
    pub fn new(seed: u64) -> Self {
        Self {
            rng: RandGenerator::new(seed),
            runtime_id: COUNTERPARTY_RUNTIME_ID,
            ..Default::default()
        }
    }

    #[cfg(feature = "log")]
    pub fn log(&mut self, log: Log) {
        self.logs.push(log);
    }

    #[cfg(feature = "log")]
    pub fn dump_logs(&mut self) -> Vec<Log> {
        let mut logs = vec![];
        logs.append(&mut self.logs);
        logs
    }

    #[cfg(not(feature = "log"))]
    pub fn log(&self) {}

    pub fn battle_running(&self) -> bool {
        self.battle_level.is_some()
    }

    pub fn battle_level(&self) -> Option<&EnemyLevel> {
        self.battle_level.as_ref()
    }

    pub fn clear_battle_level(&mut self) {
        self.battle_level = None;
    }

    #[cfg(feature = "debug")]
    pub fn get_history_signals(&mut self) -> &BTreeMap<u8, Vec<Signal>> {
        &self.signal_history
    }

    pub fn signal(&mut self, signal: Signal) {
        self.signals.push(signal);
    }

    pub fn pop_signal(&mut self) -> Option<Signal> {
        if self.signals.is_empty() {
            return None;
        }
        Some(self.signals.remove(0))
    }

    pub fn applied_signal(&mut self, signal: Signal) {
        self.signal_history
            .entry(self.current_battle_round)
            .or_default()
            .push(signal);
    }

    pub fn dump_history_delete_runtimes(&mut self) -> Vec<u16> {
        let mut delete_history = vec![];
        delete_history.append(&mut self.delete_runtime_history);
        delete_history
    }

    pub fn clear(&mut self) {
        self.current_battle_round = 0;
        self.battle_level = None;
        self.signals.clear();
        self.signal_history.clear();
        self.delete_runtime_history.clear();
    }
}
