use anchor_lang::prelude::*;

use anchor_lang::solana_program::{program::invoke, system_instruction};
use anchor_spl::token::{transfer, Transfer};
use num_bigint::BigInt;
use num_traits::cast::ToPrimitive;

use crate::errors::ErrorCodes;
use crate::instructions::swap::Swap;

pub fn swap_sol_to_fixed_token(ctx: Context<Swap>, tokens_out: u64, max_sol_in: u64) -> Result<()> {
    let global = &ctx.accounts.global;

    /* fail if global hasnt been initialized */
    require!(global.initialized, ErrorCodes::GlobalUninitialized);

    /* fail if provided fee_recipient does not match global state */
    require!(
        ctx.accounts.fee_recipient.key() == global.fee_recipient,
        ErrorCodes::FeeRecipientInvalid,
    );

    let bonding_curve = &ctx.accounts.bonding_curve;
    bonding_curve.print()?;

    /* fail if the bonding curve is already complete */
    require!(!bonding_curve.complete, ErrorCodes::BondingCurveComplete);

    /* fail if tokens_out is greater than real_token_reserves */
    require!(
        tokens_out <= bonding_curve.real_token_reserves,
        ErrorCodes::InsufficientReserves
    );

    let tokens_out_bigint = BigInt::from(tokens_out);

    let virtual_sol_reserves = BigInt::from(bonding_curve.virtual_sol_reserves);
    let virtual_token_reserves = BigInt::from(bonding_curve.virtual_token_reserves);

    let mul_result = &virtual_token_reserves * &virtual_sol_reserves;
    let sub_result = &virtual_token_reserves - tokens_out_bigint;
    let div_result = mul_result / sub_result;
    let sol_in = div_result - &virtual_sol_reserves;
    let sol_in_u64 = sol_in.to_u64().unwrap();

    /* make sure sol_in_u64 is less than max_sol_in */
    require!(sol_in_u64 <= max_sol_in, ErrorCodes::SlippageExceeded);

    msg!(
        "initial quote: {} tokens for {} lamports",
        &tokens_out,
        &sol_in_u64,
    );

    /* calculate fees */
    let trade_fee = (sol_in_u64 * global.fee_basis_points) / 10_000;
    msg!("trade fee: {} lamports", trade_fee);

    /* should have a check to make sure buy does not exceed max per wallet here */

    /* transfer sol to bonding curve */
    invoke(
        &system_instruction::transfer(
            &ctx.accounts.user.key(),
            &ctx.accounts.bonding_curve.key(),
            sol_in_u64,
        ),
        &[
            ctx.accounts.user.to_account_info(),
            ctx.accounts.bonding_curve.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
    )?;

    /* transfer fees to fee recipient */
    invoke(
        &system_instruction::transfer(
            &ctx.accounts.user.key(),
            &ctx.accounts.fee_recipient.key(),
            trade_fee,
        ),
        &[
            ctx.accounts.user.to_account_info(),
            ctx.accounts.fee_recipient.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
    )?;

    /* transfer tokens to buyer */
    let mint_key = ctx.accounts.mint.key();
    let seeds = &[
        mint_key.as_ref(),
        b"bonding_curve",
        &[ctx.bumps.bonding_curve],
    ];

    transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.associated_bonding_curve.to_account_info(),
                to: ctx.accounts.associated_user.to_account_info(),
                authority: ctx.accounts.bonding_curve.to_account_info(),
            },
            &[&seeds[..]],
        ),
        tokens_out,
    )?;

    /* update bonding curve info */
    let bonding_curve = &mut ctx.accounts.bonding_curve;

    bonding_curve.virtual_token_reserves -= tokens_out;
    bonding_curve.real_token_reserves -= tokens_out;
    bonding_curve.virtual_sol_reserves += sol_in_u64; /* make sure this only includes sol spent for tokens not fees */
    bonding_curve.real_sol_reserves += sol_in_u64; /* make sure this only includes sol spent for tokens not fees */

    if bonding_curve.real_token_reserves == 0 {
        bonding_curve.complete = true;
    }

    /* transfer fees to fee recipient */

    Ok(())
}
