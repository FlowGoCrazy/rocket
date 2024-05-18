use anchor_lang::error_code;

#[error_code]
pub enum ErrorCodes {
    #[msg("slippage exceeded: output less than minimum required")]
    SlippageExceeded,

    #[msg("bonding curve complete: trading locked until migration to raydium")]
    BondingCurveComplete,

    #[msg("insufficient funds: not enough funds to complete transaction")]
    InsufficientFunds,

    #[msg("insufficient reserves: not enough funds in reserve to complete transaction")]
    InsufficientReserves,
}
