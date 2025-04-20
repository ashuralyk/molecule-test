use alloc::string::String;
use common::enum_with_display;
use serde::{Deserialize, Serialize};

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::wasm_bindgen;

enum_with_display!(
    #[cfg_attr(feature = "wasm", wasm_bindgen)]
    pub enum CardMovement {
        DeckToHand,
        GraveToHand,
        AllExileToGrave,
        AllGraveToDeck,
        RemoveFromGame,
        AllHandToExile,
        HandToExile,
        HandToGrave,
    }
);

enum_with_display!(
    #[derive(Debug, Clone, Copy, Serialize, Deserialize)]
    #[cfg_attr(feature = "wasm", wasm_bindgen)]
    pub enum LogName {
        InitPlayer,
        BattleOver,
        GameOver,
        EnterBattle,
        PlayerTurn,
        EnemyTurn,
        AddEnemy,
        RemoveEnemy,
        Damage,
        Heal,
        HpChange,
        MaxHpChange,
        AttackChange,
        DefenseChange,
        SpiritChange,
        BlockChange,
        ShieldChange,
        BuffPointChange,
        EnergyChange,
        EnergyMaxChange,
        CardCostChange,
        CardTargetChange,
        CardAwakeChange,
        CardExileChange,
        AddBuff,
        RemoveBuff,
        BuffApplied,
        AddCard,
        CardMove,
        DeckReset,
        LootCards,
        LootGold,
        AddAction,
        ClearActions,
        SelectCards,
        ActionPointChange,
    }
);

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct Log {
    pub name: LogName,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) attribute: Option<String>,
    pub value: Option<u16>,
    pub recipient: Option<u16>,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl Log {
    #[wasm_bindgen(constructor)]
    pub fn new(log: String) -> Self {
        serde_json::from_str(&log).expect("wasm log")
    }

    #[wasm_bindgen(getter)]
    pub fn attribute(&self) -> Option<String> {
        self.attribute.clone()
    }

    #[wasm_bindgen(setter)]
    pub fn set_attribute(&mut self, value: Option<String>) {
        self.attribute = value;
    }
}
