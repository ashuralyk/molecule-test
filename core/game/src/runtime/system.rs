use alloc::{collections::BTreeMap, vec, vec::Vec};
use common::{
    card::Card, enemy::EnemyLevel, enum_with_display, hardcoded::DNA, player::roulette_player,
};
use database::PLAYER_POOL;
use serde::{Deserialize, Serialize};

use crate::{
    err, iter_stats, push_log, Context, Error, GameStatistics, RuntimeType, Signal, SignalName,
    SignalValue, BATTLE_RUNTIME_ID, PLAYER_RUNTIME_ID, SYSTEM_RUNTIME_ID,
};

#[cfg(feature = "replay")]
use common::operation::GameOperation;

enum_with_display!(
    #[derive(Clone, Copy)]
    #[cfg_attr(feature = "log", derive(serde::Serialize, serde::Deserialize))]
    pub enum CardSelectionSource {
        NoSelection,
        Deck,
        Loot,
        Grave,
        Exile,
        Hand,
    }
);

#[derive(Clone, Copy)]
#[cfg_attr(feature = "log", derive(serde::Serialize, serde::Deserialize))]
pub struct CardSelection {
    pub source: CardSelectionSource,
    pub source_runtime_id: u16,
    pub count: usize,
}

impl Default for CardSelection {
    fn default() -> Self {
        Self {
            source: CardSelectionSource::NoSelection,
            source_runtime_id: SYSTEM_RUNTIME_ID,
            count: 0,
        }
    }
}

impl CardSelection {
    pub fn from_signal(signal: &Signal) -> Result<Self, Error> {
        let source = match signal.name {
            SignalName::SelectCardFromDeck => CardSelectionSource::Deck,
            SignalName::SelectCardFromLoot => CardSelectionSource::Loot,
            SignalName::SelectCardFromGrave => CardSelectionSource::Grave,
            SignalName::SelectCardFromExile => CardSelectionSource::Exile,
            SignalName::SelectCardFromHand => CardSelectionSource::Hand,
            _ => return Err(err!(SystemInvalidCardSelection)),
        };
        let SignalValue::Positive(count) = signal.value else {
            return Err(err!(SystemInvalidCardSelection));
        };
        Ok(Self {
            source,
            source_runtime_id: signal.source_runtime_id,
            count: count as usize,
        })
    }

    pub fn wait_selection(&self, include_loot: bool) -> bool {
        if include_loot {
            !matches!(self.source, CardSelectionSource::NoSelection)
        } else {
            matches!(
                self.source,
                CardSelectionSource::Deck
                    | CardSelectionSource::Exile
                    | CardSelectionSource::Grave
                    | CardSelectionSource::Hand
            )
        }
    }

    pub fn to_signal(self, runtime_ids: Vec<u16>) -> Result<Signal, Error> {
        let name = match self.source {
            CardSelectionSource::Deck => SignalName::SelectCardFromDeck,
            CardSelectionSource::Loot => SignalName::SelectCardFromLoot,
            CardSelectionSource::Grave => SignalName::SelectCardFromGrave,
            CardSelectionSource::Exile => SignalName::SelectCardFromExile,
            CardSelectionSource::Hand => SignalName::SelectCardFromHand,
            _ => return Err(err!(SystemInvalidCardSelection)),
        };
        Ok(Signal {
            name,
            value: SignalValue::RuntimeIdArray(runtime_ids),
            transformed: false,
            source_runtime_id: SYSTEM_RUNTIME_ID,
            target_runtime_ids: vec![self.source_runtime_id],
        })
    }
}

#[derive(Serialize, Deserialize)]
pub struct PveSystemRuntime {
    ctx: Context,
    game_over: bool,
    gold: u16,
    action_point: u16,
    #[serde(skip)]
    #[serde(default)]
    card_selection: CardSelection,
    #[serde(skip)]
    #[serde(default)]
    _loot_cards: BTreeMap<u16, Card>,
    #[serde(skip)]
    #[serde(default)]
    _loot_count: usize,
    #[cfg(feature = "replay")]
    #[serde(skip)]
    #[serde(default)]
    operations: Vec<GameOperation>,
}

impl PveSystemRuntime {
    pub fn new(seed: u64) -> Result<Self, Error> {
        Ok(Self {
            ctx: Context::new(seed),
            game_over: false,
            card_selection: CardSelection::default(),
            gold: 0,
            _loot_cards: BTreeMap::new(),
            _loot_count: 0,
            action_point: 0,
            #[cfg(feature = "replay")]
            operations: vec![],
        })
    }

    pub fn game_over(&self) -> bool {
        self.game_over
    }

    pub fn start_game(
        &mut self,
        player_level: u8,
        player_action_point: u16,
        gameplay_cards: Vec<DNA>,
    ) -> Result<(), Error> {
        #[cfg(feature = "replay")]
        self.operations.push(GameOperation::StartGame);
        if self.card_selection.wait_selection(false) {
            return Err(err!(SystemCardSelectionWait));
        }
        let player = roulette_player(PLAYER_POOL, player_level, self.ctx.rng.next_u64())
            .ok_or(err!(PlayerLevelNotFound(player_level)))?;
        self.run_signal(Signal {
            name: SignalName::InitPlayer,
            value: SignalValue::Player(player, player_action_point, gameplay_cards),
            transformed: false,
            source_runtime_id: SYSTEM_RUNTIME_ID,
            target_runtime_ids: vec![SYSTEM_RUNTIME_ID],
        })?;
        let card_runtime_ids = self.ctx.runtimes.collect_runtime_ids(RuntimeType::Card);
        self.run_signal(Signal {
            name: SignalName::AddCard,
            value: SignalValue::Empty,
            transformed: false,
            source_runtime_id: SYSTEM_RUNTIME_ID,
            target_runtime_ids: card_runtime_ids,
        })
    }

    pub fn start_battle(&mut self, enemy_level: EnemyLevel, version: u8) -> Result<(), Error> {
        #[cfg(feature = "replay")]
        self.operations
            .push(GameOperation::StartBattle(enemy_level));
        if self.card_selection.wait_selection(false) {
            return Err(err!(SystemCardSelectionWait));
        }
        if enemy_level == EnemyLevel::Easy {
            iter_stats!(self.ctx, easy_mode_count, 1);
        } else if enemy_level == EnemyLevel::Normal {
            iter_stats!(self.ctx, normal_mode_count, 1);
        } else if enemy_level == EnemyLevel::Hard {
            iter_stats!(self.ctx, hard_mode_count, 1);
        }
        self.run_signal(Signal {
            name: SignalName::InitBattle,
            value: SignalValue::EnemyLevel(enemy_level, version),
            transformed: false,
            source_runtime_id: SYSTEM_RUNTIME_ID,
            target_runtime_ids: vec![BATTLE_RUNTIME_ID, SYSTEM_RUNTIME_ID],
        })
    }

    pub fn round_over(&mut self) -> Result<(), Error> {
        #[cfg(feature = "replay")]
        self.operations.push(GameOperation::RoundOver);
        if self.card_selection.wait_selection(false) {
            return Err(err!(SystemCardSelectionWait));
        }
        self.run_signal(Signal {
            name: SignalName::EnemyTurn,
            value: SignalValue::Empty,
            transformed: false,
            source_runtime_id: SYSTEM_RUNTIME_ID,
            target_runtime_ids: vec![BATTLE_RUNTIME_ID],
        })
    }

    pub fn spell_card(
        &mut self,
        card_runtime_id: u16,
        target_runtime_id: Option<u16>,
    ) -> Result<(), Error> {
        #[cfg(feature = "replay")]
        self.operations
            .push(GameOperation::SpellCard(card_runtime_id, target_runtime_id));
        if self.card_selection.wait_selection(false) {
            return Err(err!(SystemCardSelectionWait));
        }
        let signal = if let Some(target_runtime_id) = target_runtime_id {
            Signal {
                name: SignalName::SpellCard,
                value: SignalValue::RuntimeId(target_runtime_id),
                transformed: false,
                source_runtime_id: card_runtime_id,
                target_runtime_ids: vec![PLAYER_RUNTIME_ID],
            }
        } else {
            Signal {
                name: SignalName::SpellCard,
                value: SignalValue::Empty,
                transformed: false,
                source_runtime_id: card_runtime_id,
                target_runtime_ids: vec![PLAYER_RUNTIME_ID],
            }
        };
        self.run_signal(signal)
    }

    pub fn select_card(&mut self, runtime_ids: Vec<u16>) -> Result<(), Error> {
        #[cfg(feature = "replay")]
        self.operations
            .push(GameOperation::SelectCard(runtime_ids.clone()));
        if !self.card_selection.wait_selection(true) {
            return Err(err!(SystemCardSelectionNotWait));
        }
        if self.card_selection.count < runtime_ids.len() {
            return Err(err!(SystemCardSelectionExceeded));
        }
        let signal = self.card_selection.to_signal(runtime_ids)?;
        self.card_selection = CardSelection::default();
        self.run_signal(signal)
    }

    pub fn heal_hp(&mut self) -> Result<(), Error> {
        #[cfg(feature = "replay")]
        self.operations.push(GameOperation::HealHp);
        if self.ctx.battle_running() {
            return Err(err!(SystemBattleInProgress));
        }
        let healing = {
            let player = self.ctx.runtimes.get(&PLAYER_RUNTIME_ID)?.player()?;
            if self.action_point < player.raw.heal_action_point as u16 {
                return Err(err!(SystemInsufficientActionPoint));
            }
            self.action_point -= player.raw.heal_action_point as u16;
            // 30% of max hp
            let healing = player.raw.hp * 3 / 10;
            push_log!(self.ctx, {
                name: ActionPointChange,
                value: self.action_point,
                recipient: SYSTEM_RUNTIME_ID,
            });
            healing
        };
        self.run_signal(Signal {
            name: SignalName::ChangeRealHp,
            value: SignalValue::Positive(healing),
            transformed: false,
            source_runtime_id: SYSTEM_RUNTIME_ID,
            target_runtime_ids: vec![PLAYER_RUNTIME_ID],
        })
    }

    pub fn destroy_card(&mut self) -> Result<(), Error> {
        #[cfg(feature = "replay")]
        self.operations.push(GameOperation::DestroyCard);
        if self.ctx.battle_running() {
            return Err(err!(SystemBattleInProgress));
        }
        if self.card_selection.wait_selection(false) {
            return Err(err!(SystemCardSelectionInProgress));
        }
        let signal = Signal {
            name: SignalName::SelectCardFromDeck,
            value: SignalValue::Positive(1),
            transformed: false,
            source_runtime_id: SYSTEM_RUNTIME_ID,
            target_runtime_ids: vec![SYSTEM_RUNTIME_ID],
        };
        self.card_selection = CardSelection::from_signal(&signal)?;
        push_log!(self.ctx, {
            name: SelectCards,
            attribute: serde_json::to_string(&self.card_selection).unwrap(),
            value: self.card_selection.source_runtime_id,
            recipient: PLAYER_RUNTIME_ID,
        });
        let action_point_decrease = self
            .ctx
            .runtimes
            .get(&PLAYER_RUNTIME_ID)?
            .player()?
            .raw
            .discard_action_point;
        if self.action_point < action_point_decrease as u16 {
            return Err(err!(SystemInsufficientActionPoint));
        }
        self.action_point -= action_point_decrease as u16;
        push_log!(self.ctx, {
            name: ActionPointChange,
            value: self.action_point,
            recipient: SYSTEM_RUNTIME_ID,
        });
        Ok(())
    }

    pub fn get_gold(&self) -> u16 {
        self.gold
    }

    pub fn get_action_point(&self) -> u16 {
        self.action_point
    }

    pub fn statistics(&self) -> &GameStatistics {
        &self.ctx.statistics
    }

    #[cfg(feature = "log")]
    pub fn logs(&mut self) -> Vec<crate::Log> {
        self.ctx.dump_logs()
    }

    #[cfg(feature = "replay")]
    pub fn operations(&self) -> &Vec<GameOperation> {
        &self.operations
    }
}

impl PveSystemRuntime {
    fn transform_signal(&mut self, mut signal: Signal) -> Result<Option<Signal>, Error> {
        if signal.transformed {
            return Ok(Some(signal));
        }
        let mut transformed = false;
        let mut ids = self.ctx.runtimes.collect_runtime_ids(RuntimeType::Effect);
        while !ids.is_empty() {
            let effect = self.ctx.runtimes.remove(&ids.remove(0))?;
            let transformed_signals = effect.transform(&signal, &mut self.ctx)?;
            if !transformed_signals.is_empty() {
                transformed = true;
                push_log!(self.ctx, {
                    name: BuffApplied,
                    value: effect.runtime_id(),
                    recipient: effect.effect().unwrap().parent_runtime_id,
                });
            }
            transformed_signals.into_iter().for_each(|signal| {
                self.ctx.signal(signal);
            });
            self.ctx.runtimes.insert(effect);
        }
        if transformed {
            Ok(None)
        } else {
            signal.transformed = true;
            Ok(Some(signal))
        }
    }

    fn run_signal(&mut self, signal: Signal) -> Result<(), Error> {
        self.ctx.signal(signal);
        self.run()
    }

    fn run(&mut self) -> Result<(), Error> {
        while let Some(signal) = self.ctx.pop_signal() {
            let Some(signal) = self.transform_signal(signal)? else {
                continue;
            };
            if signal.name == SignalName::Skip {
                continue;
            }
            self.ctx.applied_signal(signal.clone());
            let mut ids = {
                let mut ids = self.ctx.runtimes.collect_runtime_ids(RuntimeType::Card);
                ids.append(&mut self.ctx.runtimes.collect_runtime_ids(RuntimeType::Enemy));
                ids.append(&mut self.ctx.runtimes.collect_runtime_ids(RuntimeType::Effect));
                ids.push(BATTLE_RUNTIME_ID);
                ids.push(PLAYER_RUNTIME_ID);
                ids.push(SYSTEM_RUNTIME_ID);
                ids
            };
            while !ids.is_empty() {
                let runtime_id = ids.remove(0);
                if runtime_id == SYSTEM_RUNTIME_ID {
                    self.run_self(&signal)?;
                } else {
                    let Ok(mut runtime) = self.ctx.runtimes.remove(&runtime_id) else {
                        continue;
                    };
                    runtime.run(&signal, &mut self.ctx)?;
                    self.ctx.runtimes.insert(runtime);
                }
            }
        }
        self.ctx
            .dump_history_delete_runtimes()
            .into_iter()
            .try_for_each(|id| {
                self.ctx.runtimes.remove(&id)?;
                Ok(())
            })?;
        if let Some(battle_level) = self.ctx.battle_level().cloned() {
            if self
                .ctx
                .runtimes
                .collect_runtimes(RuntimeType::Enemy)
                .iter()
                .map(|rt| rt.enemy())
                .collect::<Result<Vec<_>, _>>()?
                .iter()
                .all(|enemy| enemy.hp == 0)
            {
                self.ctx.clear_battle_level();
                let target_runtime_ids = {
                    let mut ids = self.ctx.runtimes.collect_runtime_ids(RuntimeType::Card);
                    ids.append(&mut self.ctx.runtimes.collect_runtime_ids(RuntimeType::Effect));
                    ids.push(SYSTEM_RUNTIME_ID);
                    ids.push(BATTLE_RUNTIME_ID);
                    ids.push(PLAYER_RUNTIME_ID);
                    ids
                };
                self.run_signal(Signal {
                    name: SignalName::BattleOver,
                    value: SignalValue::EnemyLevel(battle_level, Default::default()),
                    transformed: true,
                    source_runtime_id: SYSTEM_RUNTIME_ID,
                    target_runtime_ids,
                })?;
                let dead_enemies = self.ctx.runtimes.collect_runtime_ids(RuntimeType::Enemy);
                dead_enemies.into_iter().try_for_each(|enemy_id| {
                    self.ctx.runtimes.remove(&enemy_id)?;
                    Ok(())
                })?;
                self.ctx.clear();
            }
        }
        Ok(())
    }

    fn run_self(&mut self, _signal: &Signal) -> Result<(), Error> {
        Ok(())
    }
}
