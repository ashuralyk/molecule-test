use alloc::{collections::btree_map::BTreeMap, vec::Vec};
use serde::{Deserialize, Serialize};

use crate::{err, Context, Error, Signal};

mod battle;
mod card;
mod effect;
mod enemy;
mod player;
mod system;

pub use battle::*;
pub use card::*;
pub use effect::*;
pub use enemy::*;
pub use player::*;
pub use system::*;

pub const SYSTEM_RUNTIME_ID: u16 = 1;
pub const BATTLE_RUNTIME_ID: u16 = 2;
pub const PLAYER_RUNTIME_ID: u16 = 3;
pub const COUNTERPARTY_RUNTIME_ID: u16 = 4;

#[macro_export]
macro_rules! push_log {
    ($ctx:expr, { name: $name:ident, attribute: $attr:expr, value: $val:expr, recipient: $recp:expr, }) => {
        #[cfg(feature = "log")]
        $ctx.log($crate::Log {
            name: $crate::LogName::$name,
            attribute: Some($attr),
            value: Some($val as u16),
            recipient: Some($recp),
        });
        #[cfg(not(feature = "log"))]
        $ctx.log();
    };
    ($ctx:expr, { name: $name:ident, recipient: $recp:expr, }) => {
        #[cfg(feature = "log")]
        $ctx.log($crate::Log {
            name: $crate::LogName::$name,
            attribute: None,
            value: None,
            recipient: Some($recp),
        });
        #[cfg(not(feature = "log"))]
        $ctx.log();
    };
    ($ctx:expr, { name: $name:ident, attribute: $attr:expr, recipient: $recp:expr, }) => {
        #[cfg(feature = "log")]
        $ctx.log($crate::Log {
            name: $crate::LogName::$name,
            attribute: Some($attr),
            value: None,
            recipient: Some($recp),
        });
        #[cfg(not(feature = "log"))]
        $ctx.log();
    };
    ($ctx:expr, { name: $name:ident, value: $val:expr, recipient: $recp:expr, }) => {
        #[cfg(feature = "log")]
        $ctx.log($crate::Log {
            name: $crate::LogName::$name,
            attribute: None,
            value: Some($val as u16),
            recipient: Some($recp),
        });
        #[cfg(not(feature = "log"))]
        $ctx.log();
    };
}

#[macro_export]
macro_rules! iter_stats {
    ($ctx:expr, $member:ident, $change:expr) => {
        $ctx.statistics.$member += $change as u16;
    };
}

#[derive(Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RuntimePool {
    pool: BTreeMap<u16, Runtime>,
}

impl RuntimePool {
    pub fn insert(&mut self, runtime: Runtime) {
        self.pool.insert(runtime.runtime_id(), runtime);
    }

    pub fn get(&self, id: &u16) -> Result<&Runtime, Error> {
        self.pool.get(id).ok_or(err!(RuntimeNotSet(*id)))
    }

    pub fn remove(&mut self, id: &u16) -> Result<Runtime, Error> {
        self.pool.remove(id).ok_or(err!(RuntimeNotSet(*id)))
    }

    #[cfg(feature = "debug")]
    pub fn iter(&self) -> impl Iterator<Item = (&u16, &Runtime)> {
        self.pool.iter()
    }

    pub fn collect_runtime_ids(&self, runtime_type: RuntimeType) -> Vec<u16> {
        self.pool
            .iter()
            .filter(|(_, v)| v.runtime_type() == runtime_type)
            .map(|(k, _)| *k)
            .collect()
    }

    pub fn collect_runtimes(&self, runtime_type: RuntimeType) -> Vec<&Runtime> {
        self.pool
            .iter()
            .filter(|(_, v)| v.runtime_type() == runtime_type)
            .map(|(_, v)| v)
            .collect()
    }
}

#[derive(PartialEq, Eq)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub enum RuntimeType {
    Card,
    Enemy,
    Player,
    Effect,
    PveBattle,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub enum Runtime {
    Card(CardRuntime),
    Enemy(EnemyRuntime),
    Player(PlayerRuntime),
    Effect(EffectRuntime),
    PveBattle(PveBattleRuntime),
}

impl Runtime {
    pub fn runtime_id(&self) -> u16 {
        match self {
            Runtime::Card(card) => card.runtime_id,
            Runtime::Enemy(enemy) => enemy.runtime_id,
            Runtime::Player(player) => player.runtime_id,
            Runtime::Effect(effect) => effect.runtime_id,
            Runtime::PveBattle(_) => BATTLE_RUNTIME_ID,
        }
    }

    pub fn runtime_type(&self) -> RuntimeType {
        match self {
            Runtime::Card(_) => RuntimeType::Card,
            Runtime::Enemy(_) => RuntimeType::Enemy,
            Runtime::Player(_) => RuntimeType::Player,
            Runtime::Effect(_) => RuntimeType::Effect,
            Runtime::PveBattle(_) => RuntimeType::PveBattle,
        }
    }

    pub fn player(&self) -> Result<&PlayerRuntime, Error> {
        match self {
            Runtime::Player(player) => Ok(player),
            _ => Err(err!(InvalidRuntimeType)),
        }
    }

    pub fn enemy(&self) -> Result<&EnemyRuntime, Error> {
        match self {
            Runtime::Enemy(enemy) => Ok(enemy),
            _ => Err(err!(InvalidRuntimeType)),
        }
    }

    #[cfg(feature = "debug")]
    pub fn pve_battle(&self) -> Result<&PveBattleRuntime, Error> {
        match self {
            Runtime::PveBattle(battle) => Ok(battle),
            _ => Err(err!(InvalidRuntimeType)),
        }
    }
    pub fn run(&mut self, signal: &Signal, ctx: &mut Context) -> Result<(), Error> {
        match self {
            Self::Effect(v) => v.run(signal, ctx),
            Self::Player(v) => v.run(signal, ctx),
            Self::Card(v) => v.run(signal, ctx),
            Self::Enemy(v) => v.run(signal, ctx),
            Self::PveBattle(v) => v.run(signal, ctx),
        }
    }

    pub fn transform(&self, signal: &Signal, ctx: &mut Context) -> Result<Vec<Signal>, Error> {
        match self {
            Self::Effect(v) => v.transform(signal, ctx),
            _ => Err(err!(TransformOnlyForEffect)),
        }
    }
}
