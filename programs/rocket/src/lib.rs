use anchor_lang::prelude::*;
use anchor_lang::solana_program::entrypoint::ProgramResult;

declare_id!("8ppDaTFZgYJpPCrpxLow3Bq5HzZicQ6M63MGeXHPoEGb");

#[program]
pub mod rocket {
    use super::*;

    pub fn create(ctx: Context<Create>) -> ProgramResult {
        let bonding_curve = &mut ctx.accounts.bonding_curve;
        bonding_curve.virtual_token_reserves = 0;
        bonding_curve.virtual_sol_reserves = 0;
        bonding_curve.real_token_reserves = 1_000_000_000;
        bonding_curve.real_sol_reserves = 0;
        bonding_curve.token_total_supply = 1_000_000_000;
        bonding_curve.complete = false;

        Ok(())
    }

    pub fn buy(ctx: Context<Buy>) -> ProgramResult {
        let bonding_curve = &ctx.accounts.bonding_curve;
        msg!(
            "Virtual Token Reserves: {}",
            bonding_curve.virtual_token_reserves
        );
        msg!(
            "Virtual SOL Reserves: {}",
            bonding_curve.virtual_sol_reserves
        );
        msg!("Real Token Reserves: {}", bonding_curve.real_token_reserves);
        msg!("Real SOL Reserves: {}", bonding_curve.real_sol_reserves);
        msg!("Token Total Supply: {}", bonding_curve.token_total_supply);
        msg!("Complete: {}", bonding_curve.complete);

        Ok(())
    }
}

/* CONTEXT */
#[derive(Accounts)]
pub struct Create<'info> {
    #[account(init, payer = signer, space = 8 + BondingCurve::SIZE)]
    pub bonding_curve: Account<'info, BondingCurve>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Buy<'info> {
    #[account(mut)]
    pub bonding_curve: Account<'info, BondingCurve>,
    #[account(mut)]
    pub signer: Signer<'info>,
}

/* ACCOUNTS */
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
}
