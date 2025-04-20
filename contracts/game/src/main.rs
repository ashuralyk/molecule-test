#![no_main]
#![no_std]

use alloc::vec::Vec;
use ckb_cinnabar_verifier::{
    cinnabar_main, re_exports::ckb_std, this_script_args, this_script_indices, Result, ScriptPlace,
    Verification, TREE_ROOT,
};
use ckb_std::{
    ckb_constants::Source,
    ckb_types::prelude::Unpack,
    debug,
    high_level::{load_cell_data, load_cell_type, load_cell_type_hash, load_header, QueryIter},
};
use common::hardcoded::DNA_LEN;
use types::{GameConfig, PveSession, ScriptType};

mod branches;
mod error;
mod types;

use branches::*;
use error::*;

#[derive(Default)]
struct Context {
    config: GameConfig,
    pve_session_data: Option<PveSession>,
    args_payload: Vec<u8>,
    game_seed: u64,
    gameplay_cards: Vec<[u8; DNA_LEN]>,
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
    fn verify(&mut self, name: &str, _ctx: &mut Context) -> Result<Option<&str>> {
        debug!("process: {}", name);

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
