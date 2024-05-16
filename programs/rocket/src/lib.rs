use anchor_lang::prelude::*;

declare_id!("8ppDaTFZgYJpPCrpxLow3Bq5HzZicQ6M63MGeXHPoEGb");

#[program]
pub mod rocket {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("test 123");
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
