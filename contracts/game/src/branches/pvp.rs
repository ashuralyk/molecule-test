use ckb_cinnabar_verifier::{Result, Verification};

use crate::Context;

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
