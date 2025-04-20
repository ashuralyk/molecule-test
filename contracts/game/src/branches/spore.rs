use alloc::vec::Vec;

use ckb_cinnabar_verifier::{
    re_exports::ckb_std::{
        self,
        high_level::{load_cell, load_cell_data},
    },
    Result, Verification,
};
use ckb_std::{ckb_constants::Source, debug, high_level::QueryIter};
use common::{contract::SporeData, hardcoded::DEFAULT_GAMEPLAY_CARDS};

use crate::{Context, ScriptError};

fn check_spore_cards(ctx: &mut Context, check_lockup: bool) -> Result<()> {
    // In-game cards should all be locked by the type_id of PVE session from Output
    let provided_dna_set = QueryIter::new(load_cell, Source::Output)
        .enumerate()
        .filter(|(_, cell)| {
            if check_lockup {
                let lock = cell.lock();
                if !ctx.config.is_type_burn(&lock)
                    || lock.args().raw_data().as_ref() != ctx.pve_session_type_id
                {
                    return false;
                }
            }
            let Some(type_) = cell.type_().to_opt() else {
                return false;
            };
            ctx.config.is_spore(&type_)
        })
        .map(|(i, _)| {
            let data = load_cell_data(i, Source::Output)?;
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

    Ok(())
}

#[derive(Default)]
pub struct SporeCardsRedeemChecker {}

impl Verification<Context> for SporeCardsRedeemChecker {
    fn verify(&mut self, name: &str, ctx: &mut Context) -> Result<Option<&str>> {
        debug!("process: {}", name);

        check_spore_cards(ctx, false)?;
        Ok(None)
    }
}

#[derive(Default)]
pub struct SporeCardsLockupChecker {}

impl Verification<Context> for SporeCardsLockupChecker {
    fn verify(&mut self, name: &str, ctx: &mut Context) -> Result<Option<&str>> {
        debug!("process: {}", name);

        check_spore_cards(ctx, true)?;
        Ok(None)
    }
}
