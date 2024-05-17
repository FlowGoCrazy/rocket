use anchor_lang::prelude::*;
use anchor_lang::solana_program::entrypoint::ProgramResult;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};

declare_id!("8ppDaTFZgYJpPCrpxLow3Bq5HzZicQ6M63MGeXHPoEGb");

/* SEEDS */
const BONDING_CURVE: &str = "bonding_curve";

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
    #[account(
        init,
        payer = signer,
        mint::decimals = 6,
        mint::authority = mint_authority,
    )]
    pub mint: Account<'info, Mint>,

    /// CHECK: throwaway account
    pub mint_authority: UncheckedAccount<'info>,

    #[account(
        init,
        payer = signer,
        seeds = [
            mint.key().as_ref(),
            BONDING_CURVE.as_bytes(),
        ],
        space = 8 + BondingCurve::SIZE,
        bump,
    )]
    pub bonding_curve: Account<'info, BondingCurve>,

    #[account(
        init,
        payer = signer,
        associated_token::mint = mint,
        associated_token::authority = bonding_curve,
    )]
    pub associated_bonding_curve: Account<'info, TokenAccount>,

    #[account(mut)]
    pub signer: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct Buy<'info> {
    #[account(mut)]
    pub mint: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [
            mint.key().as_ref(),
            BONDING_CURVE.as_bytes(),
        ],
        bump,
    )]
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
