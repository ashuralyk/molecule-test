use ckb_cinnabar_verifier::{
    re_exports::ckb_std::{self},
    Result, Verification,
};
use ckb_std::debug;

use crate::Context;

fn check_spore_cards(_ctx: &mut Context, _check_lockup: bool) -> Result<()> {
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
