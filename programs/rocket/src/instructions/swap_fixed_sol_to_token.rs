use anchor_lang::prelude::*;

use anchor_lang::solana_program::{program::invoke, system_instruction};
use anchor_spl::token::{transfer, Transfer};
use num_bigint::BigInt;
use num_traits::cast::ToPrimitive;
use std::cmp;

use crate::errors::ErrorCodes;
use crate::instructions::swap::Swap;

pub fn swap_fixed_sol_to_token(ctx: Context<Swap>, sol_in: u64, min_tokens_out: u64) -> Result<()> {
    let bonding_curve = &ctx.accounts.bonding_curve;
    bonding_curve.print()?;

    /* fail if the bonding curve is already complete */
    if bonding_curve.complete {
        return err!(ErrorCodes::BondingCurveComplete);
    }

    /* fail if min_tokens_out is greater than real_token_reserves */
    if min_tokens_out > bonding_curve.real_token_reserves {
        return err!(ErrorCodes::InsufficientReserves);
    }

    let buy_sol_amount = BigInt::from(sol_in);

    let real_token_reserves = BigInt::from(bonding_curve.real_token_reserves);
    let virtual_sol_reserves = BigInt::from(bonding_curve.virtual_sol_reserves);
    let virtual_token_reserves = BigInt::from(bonding_curve.virtual_token_reserves);

    let mul_result = &virtual_sol_reserves * &virtual_token_reserves;
    let add_result = &virtual_sol_reserves + buy_sol_amount;
    let div_result = (mul_result / add_result) + 1;
    let sub_result = &virtual_token_reserves - div_result;
    let tokens_out = cmp::min(&sub_result, &real_token_reserves);

    /* set tokens out to either the quote or remaining token reserves, making sure it cant be higher than the amount remaining */
    let tokens_out_u64 = tokens_out.to_u64().unwrap();
    let tokens_out_u64 = std::cmp::min(tokens_out_u64, bonding_curve.real_token_reserves);

    /* make sure tokens_out_u64 is more than min_tokens_out */
    if tokens_out_u64 < min_tokens_out {
        return err!(ErrorCodes::SlippageExceeded);
    }

    /* should calculate fees here too */

    msg!(
        "initial quote: {} lamports for {} tokens",
        &sol_in,
        &tokens_out,
    );

    /* should have a check to make sure buy does not exceed max per wallet here */

    /* transfer sol to bonding curve */
    invoke(
        &system_instruction::transfer(
            &ctx.accounts.user.key(),
            &ctx.accounts.bonding_curve.key(),
            sol_in,
        ),
        &[
            ctx.accounts.user.to_account_info(),
            ctx.accounts.bonding_curve.to_account_info(),
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
        tokens_out_u64,
    )?;

    /* update bonding curve info */
    let bonding_curve = &mut ctx.accounts.bonding_curve;

    bonding_curve.virtual_token_reserves -= tokens_out_u64;
    bonding_curve.real_token_reserves -= tokens_out_u64;
    bonding_curve.virtual_sol_reserves += sol_in; /* make sure this only includes sol spent for tokens not fees */
    bonding_curve.real_sol_reserves += sol_in; /* make sure this only includes sol spent for tokens not fees */

    if bonding_curve.real_token_reserves == 0 {
        bonding_curve.complete = true;
    }

    /* transfer fees to fee recipient */

    Ok(())
}
