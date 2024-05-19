use anchor_lang::prelude::*;

#[account]
pub struct UserRef {
    pub owner: Pubkey, /* 32 bytes */
    pub balance: u64,  /* 8 bytes */
}

impl UserRef {
    pub const SIZE: usize = 32 + 8;

    pub fn print(&self) -> Result<()> {
        msg!("Owner: {}", self.owner);
        msg!("Balance: {}", self.balance);

        Ok(())
    }
}
