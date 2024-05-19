use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{transfer, Mint, Token, TokenAccount, Transfer},
};

use crate::constants::ADMIN;
use crate::errors::ErrorCodes;
use crate::state::bonding_curve::BondingCurve;

pub fn admin_withdraw(ctx: Context<AdminWithdraw>) -> Result<()> {
    let bonding_curve = &ctx.accounts.bonding_curve;

    /* fail if bonding curve isnt complete */
    require!(bonding_curve.complete, ErrorCodes::BondingCurveIncomplete);

    /* transfer tokens to admin */
    let mint_key = ctx.accounts.mint.key();
    let seeds = &[
        mint_key.as_ref(),
        b"bonding_curve",
        &[ctx.bumps.bonding_curve],
    ];

    /* get token balance */
    let associated_bonding_curve_account_info =
        &ctx.accounts.associated_bonding_curve.to_account_info();
    let associated_bonding_curve: TokenAccount =
        TokenAccount::try_deserialize(&mut &**associated_bonding_curve_account_info.data.borrow())?;
    msg!("token balance: {}", associated_bonding_curve.amount);

    transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.associated_bonding_curve.to_account_info(),
                to: ctx.accounts.associated_admin.to_account_info(),
                authority: ctx.accounts.bonding_curve.to_account_info(),
            },
            &[&seeds[..]],
        ),
        1, /* change later */
    )?;

    /* transfer sol to admin */
    msg!("lamport balance: {}", bonding_curve.real_sol_reserves);

    require!(
        **ctx
            .accounts
            .bonding_curve
            .to_account_info()
            .try_borrow_lamports()?
            >= bonding_curve.real_sol_reserves,
        ErrorCodes::InsufficientReserves
    );
    **ctx
        .accounts
        .bonding_curve
        .to_account_info()
        .try_borrow_mut_lamports()? -= bonding_curve.real_sol_reserves;
    **ctx
        .accounts
        .admin
        .to_account_info()
        .try_borrow_mut_lamports()? += bonding_curve.real_sol_reserves;

    /* update bonding curve info */
    let bonding_curve = &mut ctx.accounts.bonding_curve;
    bonding_curve.virtual_token_reserves = 0;
    bonding_curve.real_token_reserves = 0;
    bonding_curve.virtual_sol_reserves = 0;
    bonding_curve.real_sol_reserves = 0;
    bonding_curve.complete = true;

    Ok(())
}

#[derive(Accounts)]
pub struct AdminWithdraw<'info> {
    #[account(mut)]
    pub mint: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [
            mint.key().as_ref(),
            b"bonding_curve",
        ],
        bump,
    )]
    pub bonding_curve: Account<'info, BondingCurve>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = bonding_curve,
    )]
    pub associated_bonding_curve: Account<'info, TokenAccount>,

    #[account(mut, address = ADMIN)]
    pub admin: Signer<'info>,

    #[account(
        init_if_needed, /* safe from re-initialization attack because this is an admin only instruction */
        payer = admin,
        associated_token::mint = mint,
        associated_token::authority = admin,
    )]
    pub associated_admin: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}
