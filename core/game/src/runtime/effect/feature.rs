use alloc::{vec, vec::Vec};

use crate::{
    runtime::effect::{default_transform, EffectRuntime, Feature},
    Context, Error, Signal, SignalName, SignalValue,
};

/// Power up the damage caused by the owner of card to any enemies
///
/// @zh 指定回合内增加卡牌伤害
#[derive(Default)]
pub struct ExtraDamage {}

impl Feature for ExtraDamage {
    fn transform(
        &self,
        signal: &Signal,
        effect: &EffectRuntime,
        ctx: &mut Context,
    ) -> Result<Vec<Signal>, Error> {
        if signal.name != SignalName::ChangeHp {
            return Ok(vec![]);
        }
        let SignalValue::Negative(damage) = signal.value else {
            return Ok(vec![]);
        };
        let damage = damage.saturating_add(effect.value as u16);
        default_transform(
            vec![Signal {
                name: SignalName::ChangeHp,
                value: SignalValue::Negative(damage),
                transformed: true,
                source_runtime_id: effect.runtime_id,
                target_runtime_ids: signal.target_runtime_ids.clone(),
            }],
            signal,
            effect,
            ctx,
        )
    }
}
