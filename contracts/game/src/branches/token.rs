use ckb_cinnabar_verifier::{re_exports::ckb_std, Result, Verification};
use ckb_std::{
    ckb_constants::Source,
    debug,
    high_level::{
        load_cell, load_cell_data, load_cell_type, load_script, load_script_hash, QueryIter,
    },
};
use common::hardcoded::TOKEN_DECIMAL;

use crate::{Context, ScriptError, ScriptType};

#[derive(Default)]
pub struct CreateTokenIssuerCell {}

impl Verification<Context> for CreateTokenIssuerCell {
    fn verify(&mut self, name: &str, ctx: &mut Context) -> Result<Option<&str>> {
        debug!("process: {}", name);

        // Token issuer has empty args payload
        if !ctx.args_payload.is_empty() {
            return Err(ScriptError::BrokenTokenIssueArgs.into());
        }

        // Must be paired with the game global cell
        let this_code_hash = load_script()?.code_hash();
        let game_global_count = QueryIter::new(load_cell, Source::Output)
            .filter(|cell| {
                let Some(type_) = cell.type_().to_opt() else {
                    return false;
                };
                type_.code_hash() == this_code_hash
                    && type_.args().raw_data().first() == Some(&ScriptType::GameData.into())
            })
            .count();
        if game_global_count != 1 {
            return Err(ScriptError::IssuerGlobalNotPaired.into());
        }

        Ok(None)
    }
}

#[derive(Default)]
pub struct CheckTokenIssuePattern {}

impl CheckTokenIssuePattern {
    fn calculate_token_amount_from(ctx: &Context, source: Source) -> Result<u128> {
        let xudt_args = load_script_hash()?;
        let amount = QueryIter::new(load_cell_type, source)
            .enumerate()
            .filter_map(|(i, type_)| {
                let type_ = type_?;
                if ctx.config.is_xudt(&type_) && type_.args().raw_data()[..32] == xudt_args {
                    let data = load_cell_data(i, source).unwrap();
                    if data.len() < 16 {
                        return None;
                    }
                    Some(u128::from_le_bytes(data[0..16].try_into().unwrap()))
                } else {
                    None
                }
            })
            .sum::<u128>();
        Ok(amount)
    }
}

impl Verification<Context> for CheckTokenIssuePattern {
    fn verify(&mut self, name: &str, ctx: &mut Context) -> Result<Option<&str>> {
        debug!("process: {}", name);

        // Token issue pattern must be valid with the game global cell transfer
        let Some((old_game_data, _)) = Context::game_data_from(Source::Input)? else {
            return Err(ScriptError::BadGameGlobalIterationMode.into());
        };
        let Some((new_game_data, _)) = Context::game_data_from(Source::Output)? else {
            return Err(ScriptError::BadGameGlobalIterationMode.into());
        };

        // Calculate token issue amount
        let old_token_amount = Self::calculate_token_amount_from(ctx, Source::Input)?;
        let new_token_amount = Self::calculate_token_amount_from(ctx, Source::Output)?;
        let token_issue_amount = new_token_amount.saturating_sub(old_token_amount);

        // The issued token should be equal to the rising amount of hunted gold
        let expected_token_issue_amount = new_game_data
            .pve_hunted_gold
            .saturating_sub(old_game_data.pve_hunted_gold);
        if token_issue_amount != (expected_token_issue_amount as u128 * TOKEN_DECIMAL) {
            return Err(ScriptError::InvalidTokenIssueAmount.into());
        }

        Ok(None)
    }
}
