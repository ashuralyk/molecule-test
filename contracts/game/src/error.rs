use ckb_cinnabar_verifier::{define_errors, CUSTOM_ERROR_START};

define_errors!(
    ScriptError,
    {
        UnknownScriptType = CUSTOM_ERROR_START,
        UnknownOperation,
        ScriptArgsUnexpected,
        TipHeaderNotSet,
        HeaderNotSet,
        WitnessOutputTypeNotSet,
        WitnessInputTypeNotSet,
        PlayerLevelOutOfRange,
        PvePaymentNotEnough,

        BadGameGlobalInitMode,
        BadGameGlobalIterationMode,
        BadTokenIssueMode,
        BadPveCreationMode,
        BadPveSettlementMode,
        BadPveUpdateMode,
        BadRedeemMode,
        BadPvpSettlementMode,
        BadSporeLockupMode,

        BrokenGameGlobalMolecule,
        BrokenTokenIssueArgs,
        BrokenGlobalDataArgs,
        BrokenSporeDataMolecule,
        BrokenPveSessionMolecule,
        BrokenPveSessionMaterialsMolecule,
        BrokenOperationsBytes,

        IssuerGlobalNotPaired,
        PveSessionMustBeTyped,
        PvpSessionNotXudtTyped,
        RedeemSporeTypeNotFound,
        RedeemPeriodNotEnough,
        SporeCannotBeBurned,
        GameDataNotFound,
        GameDataUnexpected,
        GameplaySporeNotInCelldep,
        GameplaySporeClusterIdUnexpected,
        GameplaySporeDnaUnexpected,
        GameDataUnexpectedChanged,
        GameplayDataUnexpectedChanged,
        ActionPointOverflow,
        ActionPointUnexpectedChanged,
        GlobalDataNotInCelldep,
        GlobalOwnerProxyNotFound,
        InvalidTokenIssueAmount,
        MaterialHashMismatch,
        CardsDnaSetMismatchFromCelldep,
        CardsDnaSetMismatchFromDefault,
    }
);
