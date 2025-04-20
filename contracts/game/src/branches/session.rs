use alloc::{vec, vec::Vec};
use core::cmp::min;

use ckb_cinnabar_verifier::{
    calc_blake2b_hash,
    re_exports::ckb_std::{
        self,
        ckb_types::prelude::Entity,
        high_level::{load_cell_capacity, load_cell_type_hash},
    },
    Error, Result, Verification,
};
use ckb_std::{
    ckb_constants::Source,
    ckb_types::prelude::Unpack,
    debug,
    high_level::{load_cell, load_cell_data, load_script, load_witness_args, QueryIter},
};
use common::{
    contract::PveSessionMaterials,
    hardcoded::MAX_ACTION_POINT,
    operation::{GameOperation, GameOperationSet},
};
use game_core::PveSystemRuntime;

use crate::{types::PveSession, Context, ScriptError, ScriptType};

#[derive(Default)]
pub struct AnalyzeIteration {}

impl Verification<Context> for AnalyzeIteration {
    fn verify(&mut self, name: &str, _: &mut Context) -> Result<Option<&str>> {
        debug!("process: {}", name);

        let Some((old_game_data, _)) = Context::game_data_from(Source::Input)? else {
            return Err(ScriptError::BadGameGlobalIterationMode.into());
        };
        let Some((new_game_data, _)) = Context::game_data_from(Source::Output)? else {
            return Err(ScriptError::BadGameGlobalIterationMode.into());
        };
        if new_game_data.action_point > MAX_ACTION_POINT {
            return Err(ScriptError::ActionPointOverflow.into());
        }

        // Pattern 0: Pay CKB for charging action point
        if old_game_data.action_point_charge_mode(&new_game_data) {
            return Ok("ActionPointCharge".into());
        }

        // Pattern 1: Create a new PVE game session cell
        if old_game_data.pve_session_create_mode(&new_game_data) {
            return Ok("PveSessionCreate".into());
        }

        // Pattern 2: Burn PVE game session cell to resolve settlement
        if old_game_data.pve_session_settlement_mode(&new_game_data) {
            return Ok("PveSessionBurn".into());
        }

        // Pattern 3: Burn PVP game session cell to resolve settlement
        if old_game_data.pvp_session_settlement_mode(&new_game_data) {
            return Ok("PvpSessionResolve".into());
        }

        Err(ScriptError::BadGameGlobalIterationMode.into())
    }
}

#[derive(Default)]
pub struct ActionPointCharge {}

impl Verification<Context> for ActionPointCharge {
    fn verify(&mut self, name: &str, ctx: &mut Context) -> Result<Option<&str>> {
        debug!("process: {}", name);

        // Game data cannot be changed except the action point
        let Some((old_game_data, old_index)) = Context::game_data_from(Source::Input)? else {
            return Err(ScriptError::BadGameGlobalIterationMode.into());
        };
        let Some((new_game_data, _)) = Context::game_data_from(Source::Output)? else {
            return Err(ScriptError::BadGameGlobalIterationMode.into());
        };
        if !old_game_data.pve_equal(&new_game_data) || !old_game_data.pvp_equal(&new_game_data) {
            return Err(ScriptError::GameDataUnexpectedChanged.into());
        }

        // Calculate and check action point plus purchasing
        let action_point_block_step = ctx.action_point_block_step(old_index)?;
        let action_point_ckb_step = ctx.action_point_ckb_step()?;
        let action_point_step = action_point_block_step + action_point_ckb_step;
        debug!("action_point increased: {}", action_point_step);
        let expected_action_point = min(
            old_game_data.action_point + action_point_step,
            MAX_ACTION_POINT,
        );
        if new_game_data.action_point != expected_action_point {
            return Err(ScriptError::ActionPointUnexpectedChanged.into());
        }

        Ok(None)
    }
}

#[derive(Default)]
pub struct PveSessionCreate {}

impl Verification<Context> for PveSessionCreate {
    fn verify(&mut self, name: &str, ctx: &mut Context) -> Result<Option<&str>> {
        debug!("process: {}", name);

        // Prepare parameters checker
        let (code_hash, hash_type, script_hash) = {
            let script = load_script()?;
            (
                script.code_hash(),
                script.hash_type(),
                script.calc_script_hash(),
            )
        };
        let mut pve_args = vec![ScriptType::PveSession.into()];
        pve_args.extend(script_hash.raw_data().to_vec());

        // The new PVE session cell should be created at once
        let mut pve_session_index = QueryIter::new(load_cell, Source::Output)
            .enumerate()
            .filter_map(|(i, cell)| {
                if cell.type_().is_none() {
                    return None;
                }
                let lock = cell.lock();
                if lock.code_hash() == code_hash
                    && lock.hash_type() == hash_type
                    && lock.args().raw_data().as_ref() == pve_args
                {
                    Some(i)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        if pve_session_index.len() != 1 {
            return Err(ScriptError::BadPveCreationMode.into());
        }

        // Prepare session information
        let session_index = pve_session_index.remove(0);
        let session_data = load_cell_data(session_index, Source::Output)?;
        let pve_session_data: PveSession = serde_molecule::from_slice(&session_data, false)
            .map_err(|_| ScriptError::BrokenPveSessionMolecule)?;
        ctx.pve_session_type_id = load_cell_type_hash(session_index, Source::Output)?.unwrap();

        // Any positive payment should be placed into protocol payee
        let payment_threshold = ctx
            .config
            .player_level_cost(pve_session_data.player_level)
            .map_err(|_| ScriptError::PlayerLevelOutOfRange)?;
        if payment_threshold > 0 {
            let payee_received = QueryIter::new(load_cell, Source::Output)
                .enumerate()
                .filter(|(_, output)| {
                    output.type_().is_none()
                        && ctx
                            .config
                            .protocol_payee_scripts
                            .contains(&output.lock().as_bytes().to_vec())
                })
                .map(|(i, _)| load_cell_capacity(i, Source::Output).unwrap())
                .sum::<u64>();
            if payee_received < payment_threshold {
                return Err(ScriptError::PvePaymentNotEnough.into());
            }
        }

        // Game data cannot be changed except the action point
        let Some((old_game_data, old_index)) = Context::game_data_from(Source::Input)? else {
            return Err(ScriptError::BadGameGlobalIterationMode.into());
        };
        let Some((new_game_data, _)) = Context::game_data_from(Source::Output)? else {
            return Err(ScriptError::BadGameGlobalIterationMode.into());
        };
        if !old_game_data.pve_equal(&new_game_data) || !old_game_data.pvp_equal(&new_game_data) {
            return Err(ScriptError::GameDataUnexpectedChanged.into());
        }

        // Pve Session Principle:
        //
        // 1. Action point from game data should be transferred to the session data
        // 2. Action point in new game data is always ZERO
        // 3. In action point transferring, its change should also by applied by block increasing numbers
        let action_point_step = ctx.action_point_block_step(old_index)?;
        debug!("action_point increased: {}", action_point_step);
        let expected_action_point = min(
            old_game_data.action_point + action_point_step,
            MAX_ACTION_POINT,
        );
        if new_game_data.action_point != 0 || pve_session_data.action_point != expected_action_point
        {
            return Err(ScriptError::ActionPointUnexpectedChanged.into());
        }

        // Check the cards dna set hash, payload from witnesses
        let input_type: Vec<u8> = load_witness_args(0, Source::GroupInput)?
            .input_type()
            .to_opt()
            .ok_or(ScriptError::WitnessInputTypeNotSet)?
            .unpack();
        if calc_blake2b_hash(&[&input_type]) != pve_session_data.material_hash {
            return Err(ScriptError::MaterialHashMismatch.into());
        }
        let materials: PveSessionMaterials = serde_molecule::from_slice(&input_type, false)
            .map_err(|_| ScriptError::BrokenPveSessionMaterialsMolecule)?;
        ctx.gameplay_cards = materials.dna_collection;

        Ok(Some("SporeCardsLockupChecker"))
    }
}

#[derive(Default)]
pub struct PveSessionBurn {}

impl Verification<Context> for PveSessionBurn {
    fn verify(&mut self, name: &str, _: &mut Context) -> Result<Option<&str>> {
        debug!("process: {}", name);

        // Prepare parameters checker
        let (code_hash, hash_type, script_hash) = {
            let script = load_script()?;
            (
                script.code_hash(),
                script.hash_type(),
                script.calc_script_hash(),
            )
        };
        let mut pve_args = vec![ScriptType::PveSession.into()];
        pve_args.extend(script_hash.raw_data().to_vec());

        // Check if the PVE session cell is burning
        let pve_session_input = QueryIter::new(load_cell, Source::Input)
            .filter(|cell| {
                if cell.type_().is_none() {
                    return false;
                }
                let lock = cell.lock();
                lock.code_hash() == code_hash
                    && lock.hash_type() == hash_type
                    && lock.args().raw_data().as_ref() == pve_args
            })
            .count();
        if pve_session_input != 1 {
            return Err(ScriptError::BadPveSettlementMode.into());
        }

        // Double check if the PVE session cell is burning
        let pve_session_output = QueryIter::new(load_cell, Source::Output)
            .filter(|cell| {
                let lock = cell.lock();
                lock.code_hash() == code_hash
                    && lock.hash_type() == hash_type
                    && lock.args().raw_data().as_ref() == pve_args
            })
            .count();
        if pve_session_output != 0 {
            return Err(ScriptError::BadPveSettlementMode.into());
        }

        Ok(None)
    }
}

#[derive(Default)]
pub struct PveUpdate {}

impl PveUpdate {
    pub fn run_session_game(ctx: &mut Context) -> Result<(PveSystemRuntime, PveSessionMaterials)> {
        // Player operations should be recorded in the Witnesses
        let witness = load_witness_args(0, Source::GroupInput)
            .map_err(|_| ScriptError::WitnessOutputTypeNotSet)?;
        let output_type: Vec<u8> = witness
            .output_type()
            .to_opt()
            .ok_or(ScriptError::WitnessOutputTypeNotSet)?
            .unpack();
        let operation_set: GameOperationSet = serde_molecule::from_slice(&output_type, false)
            .map_err(|_| ScriptError::BrokenOperationsBytes)?;

        // Prepare game runtime to replay operations
        let Some(pve_session_data) = ctx.pve_session_data.as_ref() else {
            return Err(ScriptError::BadPveSettlementMode.into());
        };

        // The gameplay cards should be set in the Witnesses
        let input_type: Vec<u8> = witness
            .input_type()
            .to_opt()
            .ok_or(ScriptError::WitnessInputTypeNotSet)?
            .unpack();
        if calc_blake2b_hash(&[&input_type]) != pve_session_data.material_hash {
            return Err(ScriptError::MaterialHashMismatch.into());
        }
        let materials: PveSessionMaterials = serde_molecule::from_slice(&input_type, false)
            .map_err(|_| ScriptError::BrokenPveSessionMaterialsMolecule)?;
        ctx.gameplay_cards = materials.dna_collection.clone();

        // Prepare game runtime to replay operations
        // let mut game = if materials.archive_input.is_empty() {
        //     debug!("game mode: initialization");
        //     PveSystemRuntime::new(ctx.game_seed).map_err(|error| Error::Custom(error.into()))?
        // } else {
        //     debug!("game mode: loading");
        //     serde_molecule::from_slice(&materials.archive_input, false)
        //         .map_err(|_| ScriptError::BrokenPveSessionMaterialsMolecule)?
        // };
        let mut game =
            PveSystemRuntime::new(ctx.game_seed).map_err(|error| Error::Custom(error.into()))?;

        // Operation replay
        for operation in operation_set.operations {
            match operation {
                GameOperation::StartGame => {
                    let mut cards = vec![];
                    cards.append(&mut ctx.gameplay_cards.clone());
                    game.start_game(
                        pve_session_data.player_level,
                        pve_session_data.action_point,
                        cards,
                    )
                    .map_err(|error| Error::Custom(error.into()))?;
                }
                GameOperation::StartBattle(enemy_level) => {
                    game.start_battle(enemy_level, pve_session_data.version)
                        .map_err(|error| Error::Custom(error.into()))?;
                }
                GameOperation::RoundOver => {
                    game.round_over()
                        .map_err(|error| Error::Custom(error.into()))?;
                }
                GameOperation::SpellCard(card_runtime_id, target_runtime_id) => {
                    game.spell_card(card_runtime_id, target_runtime_id)
                        .map_err(|error| Error::Custom(error.into()))?;
                }
                GameOperation::SelectCard(card_runtime_id_set) => {
                    game.select_card(card_runtime_id_set)
                        .map_err(|error| Error::Custom(error.into()))?;
                }
                GameOperation::HealHp => {
                    game.heal_hp()
                        .map_err(|error| Error::Custom(error.into()))?;
                }
                GameOperation::DestroyCard => {
                    game.destroy_card()
                        .map_err(|error| Error::Custom(error.into()))?;
                }
            }
        }

        Ok((game, materials))
    }
}

impl Verification<Context> for PveUpdate {
    fn verify(&mut self, name: &str, ctx: &mut Context) -> Result<Option<&str>> {
        debug!("process: {}", name);

        // Run the game to get the intermediate result
        let (game, materials) = Self::run_session_game(ctx)?;
        if game.game_over() || materials.archive_output.is_empty() {
            return Err(ScriptError::BadPveUpdateMode.into());
        }

        // Check game archive iteration
        // let expected_archive_output = serde_molecule::to_vec(&game, false)
        //     .map_err(|_| ScriptError::BrokenPveSessionMaterialsMolecule)?;
        // if expected_archive_output != materials.archive_output {
        //     return Err(ScriptError::BadPveUpdateMode.into());
        // }

        Ok(None)
    }
}

#[derive(Default)]
pub struct PveSettlement {}

impl Verification<Context> for PveSettlement {
    fn verify(&mut self, name: &str, ctx: &mut Context) -> Result<Option<&str>> {
        debug!("process: {}", name);

        // Run the game to get the final result
        let (game, materials) = PveUpdate::run_session_game(ctx)?;
        if !game.game_over() || !materials.archive_output.is_empty() {
            return Err(ScriptError::BadPveSettlementMode.into());
        }

        // Settlemment Principle:
        //
        // 1. Game data should be changed by the result of game operations
        // 2. Action point won't be acumulated while the period of gameplay
        let Some((old_game_data, _)) = Context::game_data_from(Source::Input)? else {
            return Err(ScriptError::BadGameGlobalIterationMode.into());
        };
        let Some((new_game_data, _)) = Context::game_data_from(Source::Output)? else {
            return Err(ScriptError::BadGameGlobalIterationMode.into());
        };
        if !old_game_data.pvp_equal(&new_game_data) {
            return Err(ScriptError::GameDataUnexpectedChanged.into());
        }
        if new_game_data.action_point != game.get_action_point()
            || !old_game_data.statistics_equal(&new_game_data, game.statistics())
            || new_game_data.pve_hunted_gold
                != old_game_data.pve_hunted_gold + game.get_gold() as u32
        {
            return Err(ScriptError::GameplayDataUnexpectedChanged.into());
        }

        Ok(Some("SporeCardsRedeemChecker"))
    }
}

#[derive(Default)]
pub struct PvpSessionResolve {}

impl Verification<Context> for PvpSessionResolve {
    fn verify(&mut self, name: &str, _: &mut Context) -> Result<Option<&str>> {
        unimplemented!("process: {}", name);
    }
}

#[derive(Default)]
pub struct PvpSettlement {}

impl Verification<Context> for PvpSettlement {
    fn verify(&mut self, name: &str, _ctx: &mut Context) -> Result<Option<&str>> {
        unimplemented!("process: {}", name);
    }
}
