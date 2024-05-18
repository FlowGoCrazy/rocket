use anchor_lang::error_code;

#[error_code]
pub enum ErrorCodes {
    #[msg("global uninitialized: cant complete transaction until after global initialization")]
    GlobalUninitialized,

    #[msg("fee recipient invalid: mismatch between provided fee recipient and global state")]
    FeeRecipientInvalid,

    #[msg("referrer invalid: you cant refer yourself :)")]
    ReferrerInvalid,

    #[msg("slippage exceeded: output less than minimum required")]
    SlippageExceeded,

    #[msg("bonding curve complete: trading locked until migration to raydium")]
    BondingCurveComplete,

    #[msg("insufficient funds: not enough funds to complete transaction")]
    InsufficientFunds,

    #[msg("insufficient reserves: not enough funds in reserve to complete transaction")]
    InsufficientReserves,
}
