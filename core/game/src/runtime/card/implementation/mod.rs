mod attack;

pub use attack::*;

#[macro_export]
macro_rules! spelled_from_player {
    ($signal:ident, $card:ident) => {
        if !$signal.is_target($card.runtime_id)
            || $signal.name != SignalName::SpellCard
            || $card.parent_runtime_id != $signal.source_runtime_id
        {
            return Ok(vec![]);
        }
    };
    ($signal:ident, $card:ident, $signals:ident) => {
        if !$signal.is_target($card.runtime_id)
            || $signal.name != SignalName::SpellCard
            || $card.parent_runtime_id != $signal.source_runtime_id
        {
            return Ok($signals);
        }
    };
}

#[macro_export]
macro_rules! single_enemy {
    ($signal:ident, $runtimes:expr, $card:ident) => {{
        if $signal.name != SignalName::SpellCard {
            return Err($crate::err!(CardInvalidImplementationSignal(
                $card.raw.name
            )));
        }
        let SignalValue::RuntimeId(enemy_runtime_id) = $signal.value else {
            return Err($crate::err!(CardInvalidImplementationSignal(
                $card.raw.name
            )));
        };
        $runtimes.get(&enemy_runtime_id)?.enemy()?
    }};
}
