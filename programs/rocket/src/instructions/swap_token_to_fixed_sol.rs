use anchor_lang::prelude::*;

use anchor_lang::solana_program::{program::invoke, system_instruction};
use anchor_spl::token::{transfer, Transfer};
use num_bigint::BigInt;
use num_traits::cast::ToPrimitive;

use crate::errors::ErrorCodes;
use crate::instructions::swap::Swap;

pub fn swap_token_to_fixed_sol(ctx: Context<Swap>, sol_out: u64, max_tokens_in: u64) -> Result<()> {
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

    /* fail if sol_out is greater than real_sol_reserves */
    require!(
        sol_out <= bonding_curve.real_sol_reserves,
        ErrorCodes::InsufficientReserves
    );

    let sol_out_bigint = BigInt::from(sol_out);

    let virtual_sol_reserves = BigInt::from(bonding_curve.virtual_sol_reserves);
    let virtual_token_reserves = BigInt::from(bonding_curve.virtual_token_reserves);

    let mul_result = &virtual_token_reserves * &virtual_sol_reserves;
    let sub_result = &virtual_sol_reserves - sol_out_bigint;
    let div_result = mul_result / sub_result;
    let tokens_in = &div_result - &virtual_token_reserves;
    let tokens_in_u64 = tokens_in.to_u64().unwrap();

    /* make sure tokens_in_u64 is less than or equal to max_tokens_in */
    require!(tokens_in_u64 <= max_tokens_in, ErrorCodes::SlippageExceeded);

    msg!(
        "initial quote: {} tokens for {} lamports",
        &tokens_in_u64,
        &sol_out,
    );

    /* calculate fees */
    let trade_fee = (sol_out * global.fee_basis_points) / 10_000;
    msg!("trade fee: {} lamports", trade_fee);

    /* transfer tokens to bonding curve */
    transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.associated_user.to_account_info(),
                to: ctx.accounts.associated_bonding_curve.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        ),
        tokens_in_u64,
    )?;

    /* transfer fees to fee recipient */
    if trade_fee > 0 {
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
    }

    /* transfer sol from bonding curve to user */
    require!(
        **ctx
            .accounts
            .bonding_curve
            .to_account_info()
            .try_borrow_lamports()?
            >= sol_out,
        ErrorCodes::InsufficientReserves
    );
    **ctx
        .accounts
        .bonding_curve
        .to_account_info()
        .try_borrow_mut_lamports()? -= sol_out;
    **ctx
        .accounts
        .user
        .to_account_info()
        .try_borrow_mut_lamports()? += sol_out;

    /* update bonding curve info */
    let bonding_curve = &mut ctx.accounts.bonding_curve;

    bonding_curve.virtual_token_reserves += tokens_in_u64;
    bonding_curve.real_token_reserves += tokens_in_u64;
    bonding_curve.virtual_sol_reserves -= sol_out;
    bonding_curve.real_sol_reserves -= sol_out;

    Ok(())
}
