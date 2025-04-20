use alloc::vec::Vec;
use common::{card::Card, effect::Effect, enemy::EnemyLevel, hardcoded::DNA, player::Player};

#[derive(PartialEq, Clone, Copy)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub enum SignalName {
    Skip,
    InitBattle,
    BattleOver,
    InitPlayer,
    EnemyTurn,
    AddCard,
    SpellCard,
    ChangeRealHp,
    ChangeHp,
    SelectCardFromDeck,
    SelectCardFromExile,
    SelectCardFromGrave,
    SelectCardFromLoot,
    SelectCardFromHand,
}

#[allow(unused)]
#[derive(Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub enum SignalValue {
    RuntimeIdArray(Vec<u16>),
    RuntimeId(u16),
    Positive(u16),
    Negative(u16),
    Effect(Effect),
    EnemyLevel(EnemyLevel, u8),
    Player(Player, u16, Vec<DNA>),
    Card(Card),
    Empty,
}

#[derive(Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Signal {
    pub name: SignalName,
    pub value: SignalValue,
    pub transformed: bool,
    pub source_runtime_id: u16,
    pub target_runtime_ids: Vec<u16>,
}

impl Signal {
    pub fn is_target(&self, runtime_id: u16) -> bool {
        self.target_runtime_ids.contains(&runtime_id)
    }
}
