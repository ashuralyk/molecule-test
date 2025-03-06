use alloc::vec;
use core::cmp::min;

use ckb_cinnabar_verifier::{re_exports::ckb_std, Result, Verification};
use ckb_std::{
    ckb_constants::Source,
    ckb_types::prelude::Unpack,
    debug,
    high_level::{
        load_cell_lock, load_cell_type, load_cell_type_hash, load_header, load_script, QueryIter,
    },
};
use common::hardcoded::MAX_ACTION_POINT;

use crate::{Context, ScriptError, ScriptType};

#[derive(Default)]
pub struct SporeOwnershipTransfer {}

impl Verification<Context> for SporeOwnershipTransfer {
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

        // Each spore cell should be game-ownership locked
        let all_valid = QueryIter::new(load_cell_type, Source::Output)
            .enumerate()
            .filter_map(|(i, type_)| {
                if ctx.config.is_spore(&type_?) {
                    let proxy_lock = load_cell_lock(i, Source::Output).unwrap();
                    Some(proxy_lock)
                } else {
                    None
                }
            })
            .all(|lock| {
                lock.code_hash() == code_hash
                    && lock.hash_type() == hash_type
                    && lock.args().raw_data().as_ref() == spore_args
            });
        if !all_valid {
            return Err(ScriptError::BadSporeLockupMode.into());
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

        // The action point should be changed according to the block increasing numbers
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
        if new_game_data.action_point != expected_action_point {
            return Err(ScriptError::ActionPointUnexpectedChanged.into());
        }

        Ok(None)
    }
}

#[derive(Default)]
pub struct RedeemLockedSpores {}

impl Verification<Context> for RedeemLockedSpores {
    fn verify(&mut self, name: &str, ctx: &mut Context) -> Result<Option<&str>> {
        debug!("process: {}", name);

        // Global data owner must be presented in Inputs
        let Some(global_data_owner_script) = QueryIter::new(load_cell_type_hash, Source::CellDep)
            .enumerate()
            .find_map(|(i, type_hash)| {
                if type_hash.as_ref().map(AsRef::as_ref) == Some(&ctx.args_payload) {
                    Some(load_cell_lock(i, Source::CellDep).unwrap())
                } else {
                    None
                }
            })
        else {
            return Err(ScriptError::GlobalDataNotInCelldep.into());
        };
        if QueryIter::new(load_cell_lock, Source::Input)
            .all(|lock| lock != global_data_owner_script)
        {
            return Err(ScriptError::GlobalOwnerProxyNotFound.into());
        }

        // The first header in header_deps must be the latest one
        let tip_number = load_header(0, Source::HeaderDep)
            .map_err(|_| ScriptError::TipHeaderNotSet)?
            .raw()
            .number()
            .unpack();

        // Unlocking Principle:
        //
        // 1. Locked spores should wait for a sort of blocks to unlcok
        // 2. Every unlocking spores cannot be burned
        ctx.unlocked_spores
            .iter()
            .try_for_each(|(header, spore_type)| {
                // Check locking period of each unlocked spore cells
                if header.raw().number().unpack() + ctx.config.card_redeemable_blocks > tip_number {
                    return Err(ScriptError::RedeemPeriodNotEnough);
                }
                // Check the transferred spores in Outputs
                let transfer_exist = QueryIter::new(load_cell_type, Source::Output)
                    .any(|type_| type_.as_ref() == Some(spore_type));
                if !transfer_exist {
                    return Err(ScriptError::SporeCannotBeBurned);
                }
                Ok(())
            })?;

        Ok(None)
    }
}
