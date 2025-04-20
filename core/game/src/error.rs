use core::fmt::Display;

#[cfg(feature = "debug")]
use common::{card::CardName, effect::EffectName};

#[cfg(feature = "debug")]
use crate::SignalName;

macro_rules! define_error_enum {
    (
        $error_enum:ident
        { $($variant:ident $(($($field:ty),+))? $(= $code:expr)?),+ $(,)? }
    ) => {
        #[cfg(not(feature = "debug"))]
        #[derive(Debug)]
        #[repr(i8)]
        pub enum $error_enum {
            $(
                $variant $(= $code)?,
            )+
        }

        #[cfg(feature = "debug")]
        #[derive(Debug)]
        pub enum $error_enum {
            $(
                $variant $(($($field),+))?,
            )+
        }

        impl Display for $error_enum {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_fmt(format_args!("{:?}", self))
            }
        }

        #[cfg(feature = "debug")]
        impl From<$error_enum> for i8 {
            fn from(_: $error_enum) -> i8 {
                unimplemented!();
            }
        }

        #[cfg(not(feature = "debug"))]
        impl From<$error_enum> for i8 {
            fn from(error: $error_enum) -> i8 {
                error as i8
            }
        }
    };
}

#[cfg(feature = "debug")]
#[macro_export]
macro_rules! err {
    ($var:ident $(($($param:expr),+))? ) => {
        $crate::Error::$var $(($($param),+))?
    };
}

#[cfg(not(feature = "debug"))]
#[macro_export]
macro_rules! err {
    ($var:ident $(($($param:expr),+))? ) => {
        $crate::Error::$var
    };
}

define_error_enum!(
    Error {
        RuntimeNotSet(u16) = 66,
        EmptySignalTarget,
        Internal,
        InvalidRuntimeType,
        TransformOnlyForEffect,
        EffectInvalidSignal(EffectName),
        EffectInvalidOverlay(EffectName),
        EffectInvalidParameter(EffectName),
        EffectUnexpectedZeroCountdown(EffectName),
        EffectInvalidParent(EffectName),
        CardInvaidSignal(CardName, SignalName),
        CardInvalidConfig(CardName, usize),
        CardInvalidAwake(CardName),
        CardInvalidConfigExile(CardName),
        CardInvalidSignalValue(CardName),
        CardAwakeInsufficient(CardName),
        CardCannotChangeAwake(CardName),
        CardCannotSpell(CardName),
        CardInvalidImplementationSignal(CardName),
        UnexpectedCardSelectionCount(CardName),
        CardCreateFailed,
        PlayerRuntimeNotSet,
        PlayerEnergyInsufficient,
        PlayerInvalidSignalValue(SignalName),
        PlayerInvalidSignal(SignalName),
        PlayerCardNotFound,
        PlayerEffectNotFound,
        PlayerSorceryNotFound,
        PlayerHandholdExceeded,
        PlayerLevelNotFound(u8),
        PlayerExisted,
        PlayerInvalidPattern,
        EnemyRuntimeNotSet,
        EnemyExisted,
        EnemyNotDead,
        EnemyLevelNotFound,
        EnemyInvalidSignal(SignalName),
        EnemyInvalidSignalValue(SignalName),
        EnemyStateCheckFailed,
        EnemyEffectNotFound,
        EnemyActionNotSetup,
        ActionCreateFailed,
        BattleInvalidSignal,
        BattleInvalidSignalValue,
        BattleAlreadyStarted,
        BattleEffectAlreadySetup,
        BattleEffectNotFound,
        BattleNotStarted,
        SystemInvalidSignalType(SignalName),
        SystemNoBattleLoot,
        SystemExceededLoot,
        SystemInvalidSignalHistory,
        SystemInvalidCardSelection,
        SystemCardSelectionInProgress,
        SystemCardSelectionWait,
        SystemCardSelectionNotWait,
        SystemCardSelectionExceeded,
        SystemInsufficientActionPoint,
        SystemBattleInProgress,
    }
);
