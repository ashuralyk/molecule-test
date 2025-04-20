use alloc::{vec, vec::Vec};

use ckb_cinnabar_verifier::{
    calc_blake2b_hash,
    re_exports::ckb_std::{self},
    Error, Result, Verification,
};
use ckb_std::{
    ckb_constants::Source, ckb_types::prelude::Unpack, debug, high_level::load_witness_args,
};
use common::{
    contract::PveSessionMaterials,
    operation::{GameOperation, GameOperationSet},
};
use game_core::PveSystemRuntime;

use crate::{Context, ScriptError};

#[derive(Default)]
pub struct AnalyzeIteration {}

impl Verification<Context> for AnalyzeIteration {
    fn verify(&mut self, name: &str, _: &mut Context) -> Result<Option<&str>> {
        debug!("process: {}", name);

        Ok(None)
    }
}

#[derive(Default)]
pub struct ActionPointCharge {}

impl Verification<Context> for ActionPointCharge {
    fn verify(&mut self, name: &str, _ctx: &mut Context) -> Result<Option<&str>> {
        debug!("process: {}", name);

        Ok(None)
    }
}

#[derive(Default)]
pub struct PveSessionCreate {}

impl Verification<Context> for PveSessionCreate {
    fn verify(&mut self, name: &str, _ctx: &mut Context) -> Result<Option<&str>> {
        debug!("process: {}", name);

        Ok(Some("SporeCardsLockupChecker"))
    }
}

#[derive(Default)]
pub struct PveSessionBurn {}

impl Verification<Context> for PveSessionBurn {
    fn verify(&mut self, name: &str, _: &mut Context) -> Result<Option<&str>> {
        debug!("process: {}", name);

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
    fn verify(&mut self, name: &str, _ctx: &mut Context) -> Result<Option<&str>> {
        debug!("process: {}", name);

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
