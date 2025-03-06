use alloc::{vec, vec::Vec};
use core::cmp::min;

use ckb_cinnabar_verifier::{calc_blake2b_hash, re_exports::ckb_std, Result, Verification};
use ckb_std::{
    ckb_constants::Source,
    ckb_types::prelude::Unpack,
    debug,
    high_level::{
        load_cell, load_cell_data, load_header, load_script, load_witness_args, QueryIter,
    },
};
use common::{
    hardcoded::{DEFAULT_GAMEPLAY_CARDS, DNA, DNA_LEN, MAX_ACTION_POINT},
    molecule::{
        Operation, OperationType, SelectCardParameters, SpellCardParameters, SporeData,
        StartBattleParameters,
    },
};

use crate::{
    types::{PveSession, PveWitness},
    Context, ScriptError, ScriptType,
};

#[derive(Default)]
pub struct AnalyzePveGameIteration {}

impl Verification<Context> for AnalyzePveGameIteration {
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

        // Pattern 1: Transfer card spores into game proxy ownership
        if old_game_data.spore_lock_mode(&new_game_data) {
            return Ok("SporeOwnershipTransfer".into());
        }

        // Pattern 2: Create a new PVE game session cell
        if old_game_data.pve_session_create_mode(&new_game_data) {
            return Ok("PveSessionCreate".into());
        }

        // Pattern 3: Burn PVE game session cell to resolve settlement
        if old_game_data.pve_session_settlement_mode(&new_game_data) {
            return Ok("PveSessionResolve".into());
        }

        // Pattern 4: Burn PVP game session cell to resolve settlement
        if old_game_data.pvp_session_settlement_mode(&new_game_data) {
            return Ok("PvpSessionResolve".into());
        }

        Err(ScriptError::BadGameGlobalIterationMode.into())
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
                if cell.type_().is_some() {
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
        let pve_session_data: PveSession = serde_molecule::from_slice(&session_data, true)
            .map_err(|_| ScriptError::BrokenPveSessionMolecule)?;

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
        let old_header = load_header(old_index, Source::Input)
            .map_err(|_| ScriptError::HeaderNotSet)?
            .raw()
            .number();
        let tip_header = load_header(0, Source::HeaderDep)
            .map_err(|_| ScriptError::HeaderNotSet)?
            .raw()
            .number();
        let action_point_step = tip_header.unpack().saturating_sub(old_header.unpack())
            * ctx.config.action_point_per_block as u64;
        debug!("action_point increased: {}", action_point_step);
        let expected_action_point = min(
            old_game_data.action_point + action_point_step as u16,
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
        if input_type.len() % DNA_LEN != 0 {
            return Err(ScriptError::WitnessInputTypeNotSet.into());
        }
        if calc_blake2b_hash(&[&input_type]) != pve_session_data.collection_hash {
            return Err(ScriptError::CardsHashMismatch.into());
        }
        ctx.gameplay_cards = input_type
            .chunks(DNA_LEN)
            .map(|dna| {
                dna.try_into()
                    .map_err(|_| ScriptError::BrokenGameplayCards.into())
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(Some("PveSessionCardsChecker".into()))
    }
}

#[derive(Default)]
pub struct PveSessionCardsChecker {}

impl Verification<Context> for PveSessionCardsChecker {
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
        let mut spore_args = vec![ScriptType::LockedCard as u8];
        spore_args.extend(script_hash.raw_data().to_vec());

        // In-game cards should all be locked and placed in CellDeps
        let provided_dna_set = QueryIter::new(load_cell, Source::CellDep)
            .enumerate()
            .filter(|(_, cell)| {
                let lock = cell.lock();
                if lock.code_hash() != code_hash
                    || lock.hash_type() != hash_type
                    || lock.args().raw_data().as_ref() != spore_args
                {
                    return false;
                }
                let Some(type_) = cell.type_().to_opt() else {
                    return false;
                };
                ctx.config.is_spore(&type_)
            })
            .map(|(i, _)| {
                let data = load_cell_data(i, Source::CellDep)?;
                let spore_data: SporeData = serde_molecule::from_slice(&data, false)
                    .map_err(|_| ScriptError::BrokenSporeDataMolecule)?;
                if !ctx.config.is_valid_cluster(&spore_data) {
                    return Err(ScriptError::GameplaySporeClusterIdUnexpected.into());
                }
                Ok(spore_data.dna())
            })
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .map(|dna| dna.ok_or(ScriptError::GameplaySporeDnaUnexpected.into()))
            .collect::<Result<Vec<_>>>()?;

        // Check if all provided DNAs are in the witness DNA set
        for dna in &provided_dna_set {
            if let Some(index) = ctx.gameplay_cards.iter().position(|x| x == dna) {
                ctx.gameplay_cards.remove(index);
            } else {
                return Err(ScriptError::CardsDnaSetMismatchFromCelldep.into());
            }
        }

        // Check the remained cards are all in pre-defined cards collection
        let mut default_collection = DEFAULT_GAMEPLAY_CARDS.to_vec();
        for dna in &ctx.gameplay_cards {
            if let Some(index) = default_collection.iter().position(|x| x == dna) {
                default_collection.remove(index);
            } else {
                return Err(ScriptError::CardsDnaSetMismatchFromDefault.into());
            }
        }

        Ok(None)
    }
}

#[derive(Default)]
pub struct PveSessionResolve {}

impl Verification<Context> for PveSessionResolve {
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
        let pve_session = QueryIter::new(load_cell, Source::Input)
            .filter(|cell| {
                if cell.type_().is_some() {
                    return false;
                }
                let lock = cell.lock();
                lock.code_hash() == code_hash
                    && lock.hash_type() == hash_type
                    && lock.args().raw_data().as_ref() == pve_args
            })
            .count();
        if pve_session != 1 {
            return Err(ScriptError::BadPveSettlementMode.into());
        }

        Ok(None)
    }
}

#[derive(Default)]
pub struct PveSettlement {}

impl Verification<Context> for PveSettlement {
    fn verify(&mut self, name: &str, ctx: &mut Context) -> Result<Option<&str>> {
        debug!("process: {}", name);

        // Player operations should be recorded in the Witnesses
        let witness = load_witness_args(0, Source::GroupInput)
            .map_err(|_| ScriptError::WitnessOutputTypeNotSet)?;
        let output_type: Vec<u8> = witness
            .output_type()
            .to_opt()
            .ok_or(ScriptError::WitnessOutputTypeNotSet)?
            .unpack();
        let pve_witness: PveWitness = serde_molecule::from_slice(&output_type, false)
            .map_err(|_| ScriptError::BrokenPveWitnessMolecule)?;

        // Prepare game runtime to replay operations
        let Some(_pve_session_data) = ctx.pve_session_data.as_ref() else {
            return Err(ScriptError::BadPveSettlementMode.into());
        };

        // The gameplay cards should be set in the Witnesses
        let input_type: Vec<u8> = witness
            .input_type()
            .to_opt()
            .ok_or(ScriptError::WitnessInputTypeNotSet)?
            .unpack();
        let mut card_dna_set = input_type
            .chunks(DNA_LEN)
            .map(|dna| {
                dna.try_into()
                    .map_err(|_| ScriptError::BrokenGameplayCards.into())
            })
            .collect::<Result<Vec<DNA>>>()?;

        // Operation replay
        for Operation { flag, payload } in pve_witness.operations {
            match flag {
                OperationType::StartGame => {
                    let mut cards = vec![];
                    cards.append(&mut card_dna_set);
                    debug!("start game: {:?}", cards);
                }
                OperationType::StartBattle => {
                    let params: StartBattleParameters = serde_molecule::from_slice(&payload, false)
                        .map_err(|_| ScriptError::BrokenOpStartBattleMolecule)?;
                    debug!("start battle: {:?}", params);
                }
                OperationType::RoundOver => {}
                OperationType::SpellCard => {
                    let params: SpellCardParameters =
                        serde_molecule::from_slice(&payload, false)
                            .map_err(|_| ScriptError::BrokenOpSpellCardMolecule)?;
                    debug!("spell card: {:?}", params);
                }
                OperationType::SelectCard => {
                    let params: SelectCardParameters = serde_molecule::from_slice(&payload, false)
                        .map_err(|_| ScriptError::BrokenOpSelectCardMolecule)?;
                    debug!("select card: {:?}", params);
                }
                OperationType::HealHp => {
                    debug!("heal hp");
                }
                OperationType::DestroyCard => {
                    debug!("destroy card");
                }
            }
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

        Ok(None)
    }
}
