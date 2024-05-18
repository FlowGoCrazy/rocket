use anchor_lang::prelude::*;

#[account]
pub struct Global {
    pub initialized: bool, /* 1 byte */

    pub fee_recipient: Pubkey, /* 32 bytes */
    pub fee_basis_points: u64, /* 8 bytes */

    pub initial_virtual_token_reserves: u64, /* 8 bytes */
    pub initial_virtual_sol_reserves: u64,   /* 8 bytes */
    pub initial_real_token_reserves: u64,    /* 8 bytes */
    pub token_total_supply: u64,             /* 8 bytes */
}

impl Global {
    pub const SIZE: usize = 1 + 32 + 8 + 8 + 8 + 8 + 8;

    pub fn print(&self) -> Result<()> {
        msg!("Initialized: {}", self.initialized);

        msg!("Fee Recipient: {}", self.fee_recipient);
        msg!("Fee Basis Points: {}", self.fee_basis_points);

        msg!(
            "Initial Virtual Token Reserves: {}",
            self.initial_virtual_token_reserves
        );
        msg!(
            "Initial Virtual SOL Reserves: {}",
            self.initial_virtual_sol_reserves
        );
        msg!(
            "Initial Real Token Reserves: {}",
            self.initial_real_token_reserves
        );
        msg!("Token Total Supply: {}", self.token_total_supply);

        Ok(())
    }
}
