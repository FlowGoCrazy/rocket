use anchor_lang::prelude::*;

#[account]
pub struct BondingCurve {
    pub virtual_token_reserves: u64, /* 8 bytes */
    pub virtual_sol_reserves: u64,   /* 8 bytes */
    pub real_token_reserves: u64,    /* 8 bytes */
    pub real_sol_reserves: u64,      /* 8 bytes */
    pub token_total_supply: u64,     /* 8 bytes */
    pub complete: bool,              /* 1 byte */
}

impl BondingCurve {
    pub const SIZE: usize = 8 + 8 + 8 + 8 + 8 + 1;

    pub fn print(&self) -> Result<()> {
        msg!("Virtual Token Reserves: {}", self.virtual_token_reserves);
        msg!("Virtual SOL Reserves: {}", self.virtual_sol_reserves);
        msg!("Real Token Reserves: {}", self.real_token_reserves);
        msg!("Real SOL Reserves: {}", self.real_sol_reserves);
        msg!("Token Total Supply: {}", self.token_total_supply);

        msg!("Complete: {}", self.complete);

        Ok(())
    }
}
