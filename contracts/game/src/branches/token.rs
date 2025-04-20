use ckb_cinnabar_verifier::{re_exports::ckb_std, Result, Verification};
use ckb_std::debug;

use crate::Context;

#[derive(Default)]
pub struct CreateTokenIssuerCell {}

impl Verification<Context> for CreateTokenIssuerCell {
    fn verify(&mut self, name: &str, _ctx: &mut Context) -> Result<Option<&str>> {
        debug!("process: {}", name);

        Ok(None)
    }
}

#[derive(Default)]
pub struct CheckTokenIssuePattern {}

impl Verification<Context> for CheckTokenIssuePattern {
    fn verify(&mut self, name: &str, _ctx: &mut Context) -> Result<Option<&str>> {
        debug!("process: {}", name);

        Ok(None)
    }
}
