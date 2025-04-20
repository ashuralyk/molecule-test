#![no_main]
#![no_std]

use alloc::vec::Vec;
use ckb_cinnabar_verifier::{
    calc_type_id, cinnabar_main,
    re_exports::ckb_std::{self, high_level::load_cell_capacity},
    this_script_args, this_script_indices, Result, ScriptPlace, Verification, TREE_ROOT,
};
use ckb_std::{
    ckb_constants::Source,
    ckb_types::prelude::Unpack,
    debug,
    high_level::{
        load_cell, load_cell_data, load_cell_type, load_cell_type_hash, load_header, load_script,
        QueryIter,
    },
};
use common::hardcoded::DNA_LEN;
use types::{GameConfig, GameGlobal, PveSession, ScriptType};

mod branches;
mod error;
mod types;

use branches::*;
use error::*;

#[derive(Default)]
struct Context {
    config: GameConfig,
    pve_session_data: Option<PveSession>,
    pve_session_type_id: [u8; 32],
    args_payload: Vec<u8>,
    game_seed: u64,
    gameplay_cards: Vec<[u8; DNA_LEN]>,
}

impl Context {
    pub fn game_data_from(source: Source) -> Result<Option<(GameGlobal, usize)>> {
        let (code_hash, hash_type) = {
            let script = load_script()?;
            (script.code_hash(), script.hash_type())
        };

        let mut game_data_with_index = QueryIter::new(load_cell_type, source)
            .enumerate()
            .filter(|(_, type_)| {
                let Some(type_) = type_ else {
                    return false;
                };
                type_.code_hash() == code_hash
                    && type_.hash_type() == hash_type
                    && type_.args().raw_data().first() == Some(&ScriptType::GameData.into())
            })
            .map(|(index, _)| {
                let data = load_cell_data(index, source)?;
                let game_data: GameGlobal = serde_molecule::from_slice(&data, false)
                    .map_err(|_| ScriptError::BrokenGameGlobalMolecule)?;
                Ok((game_data, index))
            })
            .collect::<Result<Vec<_>>>()?;

        if game_data_with_index.is_empty() {
            Ok(None)
        } else {
            Ok(Some(game_data_with_index.remove(0)))
        }
    }

    pub fn action_point_block_step(&self, old_header_index: usize) -> Result<u16> {
        let old_header = load_header(old_header_index, Source::Input)
            .map_err(|_| ScriptError::HeaderNotSet)?
            .raw()
            .number();
        let tip_header = load_header(0, Source::HeaderDep)
            .map_err(|_| ScriptError::HeaderNotSet)?
            .raw()
            .number();
        let action_point_block_step = tip_header
            .unpack()
            .saturating_sub(old_header.unpack())
            .saturating_div(self.config.block_per_action_point as u64);
        Ok(action_point_block_step as u16)
    }

    pub fn action_point_ckb_step(&self) -> Result<u16> {
        let inputs_capacity = QueryIter::new(load_cell_capacity, Source::Input).sum::<u64>();
        let outputs_capacity = QueryIter::new(load_cell_capacity, Source::Output).sum::<u64>();
        let action_point_ckb_step = outputs_capacity
            .saturating_sub(inputs_capacity)
            .saturating_div(self.config.ckb_per_action_point);
        Ok(action_point_ckb_step as u16)
    }
}

fn game_global_update(global_type_hash: &[u8]) -> bool {
    let in_input = QueryIter::new(load_cell_type_hash, Source::Input)
        .any(|type_hash| type_hash.as_ref().map(AsRef::as_ref) == Some(global_type_hash));
    let in_output = QueryIter::new(load_cell_type_hash, Source::Output)
        .any(|type_hash| type_hash.as_ref().map(AsRef::as_ref) == Some(global_type_hash));
    in_input && in_output
}

#[derive(Default)]
struct Root {}

impl Verification<Context> for Root {
    fn verify(&mut self, name: &str, ctx: &mut Context) -> Result<Option<&str>> {
        debug!("process: {}", name);

        // First byte from args is the flag of script type
        let mut args = this_script_args()?;
        if args.is_empty() {
            return Err(ScriptError::ScriptArgsUnexpected.into());
        }
        let script_type: ScriptType = args
            .remove(0)
            .try_into()
            .map_err(|_| ScriptError::UnknownScriptType)?;
        ctx.args_payload = args;

        // Game global cell and token issuer cell are placed in Type
        let type_ins = this_script_indices(Source::Input, ScriptPlace::Type)?;
        let type_outs = this_script_indices(Source::Output, ScriptPlace::Type)?;

        // Locked spore cell and pvp/pve session cell are placed in Lock
        let lock_ins = this_script_indices(Source::Input, ScriptPlace::Lock)?;
        let lock_outs = this_script_indices(Source::Output, ScriptPlace::Lock)?;

        match script_type {
            ScriptType::GameData => {
                debug!("GameData Mode");

                if type_ins.len() > 1 || type_outs.len() > 1 {
                    return Err(ScriptError::BadGameGlobalInitMode.into());
                }

                match (type_ins.len() == 1, type_outs.len() == 1) {
                    // Creation pattern
                    (false, true) => Ok("CreateGameGlobalCell".into()),
                    // Transfer pattern
                    (true, true) => Ok("AnalyzeIteration".into()),
                    // Burn pattern
                    (true, false) => Ok(None),
                    _ => unreachable!(),
                }
            }
            ScriptType::TokenIssuer => {
                debug!("TokenIssuer Mode");

                if type_ins.len() > 1 || type_outs.len() > 1 {
                    return Err(ScriptError::BadTokenIssueMode.into());
                }

                match (type_ins.len() == 1, type_outs.len() == 1) {
                    // Creation pattern
                    (false, true) => Ok("CreateTokenIssuerCell".into()),
                    // Transfer pattern
                    (true, true) => Ok("CheckTokenIssuePattern".into()),
                    // Burn pattern
                    (true, false) => Ok(None),
                    _ => unreachable!(),
                }
            }
            ScriptType::PveSession => {
                debug!("PveSession Mode");

                let (next, data_source) = if lock_outs.len() == 1 {
                    if lock_ins.len() != 1 || game_global_update(&ctx.args_payload) {
                        return Err(ScriptError::BadPveUpdateMode.into());
                    }
                    ("PveUpdate".into(), Source::Output)
                } else {
                    if lock_ins.len() != 1
                        || !lock_outs.is_empty()
                        || !game_global_update(&ctx.args_payload)
                    {
                        return Err(ScriptError::BadPveSettlementMode.into());
                    }
                    ("PveSettlement".into(), Source::Input)
                };

                // Pve session cell has unique type script
                let type_ = load_cell_type(lock_ins[0], Source::Input)?;
                if type_.is_none() {
                    return Err(ScriptError::PveSessionMustBeTyped.into());
                }

                // Load session data
                let data = load_cell_data(lock_ins[0], data_source)?;
                let pve_session_data: types::PveSession = serde_molecule::from_slice(&data, false)
                    .map_err(|_| ScriptError::BrokenPveSessionMolecule)?;
                ctx.pve_session_data = Some(pve_session_data);

                // Set game original seed
                let pve_session_header = load_header(lock_ins[0], Source::Input)
                    .map_err(|_| ScriptError::HeaderNotSet)?;
                ctx.game_seed = pve_session_header.nonce().unpack() as u64;

                Ok(next)
            }
            ScriptType::PvpSession => {
                debug!("PvpSession Mode");

                if lock_ins.len() != 1
                    || !lock_outs.is_empty()
                    || !game_global_update(&ctx.args_payload[..32])
                    || !game_global_update(&ctx.args_payload[32..])
                {
                    return Err(ScriptError::BadPvpSettlementMode.into());
                }

                // Pvp session cell should be Xudt typed
                let type_ = load_cell_type(lock_ins[0], Source::Input)?
                    .ok_or(ScriptError::PvpSessionNotXudtTyped)?;
                if !ctx.config.is_xudt(&type_) {
                    return Err(ScriptError::PvpSessionNotXudtTyped.into());
                }
                Ok("PvpSettlement".into())
            }
        }
    }
}

#[derive(Default)]
pub struct CreateGameGlobalCell {}

impl Verification<Context> for CreateGameGlobalCell {
    fn verify(&mut self, name: &str, ctx: &mut Context) -> Result<Option<&str>> {
        debug!("process: {}", name);

        // Check the game global data is initialized
        let Some((game_data, out_index)) = Context::game_data_from(Source::Output)? else {
            return Err(ScriptError::GameDataNotFound.into());
        };
        if game_data != GameGlobal::default() {
            return Err(ScriptError::GameDataUnexpected.into());
        }

        // Must be paired with the token issuer cell
        let this_code_hash = load_script()?.code_hash();
        let token_issuer_count = QueryIter::new(load_cell, Source::Output)
            .filter(|cell| {
                let Some(type_) = cell.type_().to_opt() else {
                    return false;
                };
                type_.code_hash() == this_code_hash
                    && type_.args().raw_data().first() == Some(&ScriptType::TokenIssuer.into())
            })
            .count();
        if token_issuer_count != 1 {
            return Err(ScriptError::IssuerGlobalNotPaired.into());
        }

        // Global args payload must be its type_id calculated
        if ctx.args_payload != calc_type_id(out_index)? {
            return Err(ScriptError::BrokenGlobalDataArgs.into());
        }

        Ok(None)
    }
}

cinnabar_main!(
    Context,
    (TREE_ROOT, Root),
    ("CreateGameGlobalCell", CreateGameGlobalCell),
    ("CreateTokenIssuerCell", CreateTokenIssuerCell),
    ("CheckTokenIssuePattern", CheckTokenIssuePattern),
    ("AnalyzeIteration", AnalyzeIteration),
    ("ActionPointCharge", ActionPointCharge),
    ("PveSettlement", PveSettlement),
    ("PveUpdate", PveUpdate),
    ("PvpSettlement", PvpSettlement),
    ("PveSessionCreate", PveSessionCreate),
    ("PveSessionBurn", PveSessionBurn),
    ("SporeCardsLockupChecker", SporeCardsLockupChecker),
    ("SporeCardsRedeemChecker", SporeCardsRedeemChecker),
);
