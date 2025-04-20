use alloc::{vec, vec::Vec};

use crate::{
    single_enemy, spelled_from_player, CardRuntime, Context, Error, Implementation, Signal,
    SignalName, SignalValue,
};

/// Attack on single enemy once
///
/// @zh 攻击
#[derive(Default)]
pub struct Attack {}

impl Implementation for Attack {
    fn run(
        &mut self,
        signal: &Signal,
        card: &CardRuntime,
        ctx: &mut Context,
    ) -> Result<Vec<Signal>, Error> {
        spelled_from_player!(signal, card);
        let enemy = single_enemy!(signal, ctx.runtimes, card);
        let damage = card.param_value(0)?;
        Ok(vec![Signal {
            name: SignalName::ChangeHp,
            value: SignalValue::Negative(damage),
            transformed: false,
            source_runtime_id: card.runtime_id,
            target_runtime_ids: vec![enemy.runtime_id],
        }])
    }
}
