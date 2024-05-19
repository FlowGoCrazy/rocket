use anchor_lang::prelude::*;

use anchor_lang::solana_program::{program::invoke, system_instruction};
use anchor_spl::token::{transfer, Transfer};
use num_bigint::BigInt;
use num_traits::cast::ToPrimitive;
use std::cmp;

use crate::errors::ErrorCodes;
use crate::instructions::swap::Swap;

pub fn swap_fixed_sol_to_token(ctx: Context<Swap>, sol_in: u64, min_tokens_out: u64) -> Result<()> {
    let global = &ctx.accounts.global;

    /* fail if global hasnt been initialized */
    require!(global.initialized, ErrorCodes::GlobalUninitialized);

    /* fail if provided fee_recipient does not match global state */
    require!(
        ctx.accounts.fee_recipient.key() == global.fee_recipient,
        ErrorCodes::FeeRecipientInvalid,
    );

    /* fail if referrer is the same as user */
    require!(
        ctx.accounts.referrer.key() != ctx.accounts.user.key(),
        ErrorCodes::ReferrerInvalid
    );

    let bonding_curve = &ctx.accounts.bonding_curve;
    bonding_curve.print()?;

    /* fail if the bonding curve is already complete */
    require!(!bonding_curve.complete, ErrorCodes::BondingCurveComplete);

    /* fail if the bonding curve is already complete */
    require!(!bonding_curve.complete, ErrorCodes::BondingCurveComplete);

    /* fail if min_tokens_out is greater than real_token_reserves */
    require!(
        min_tokens_out <= bonding_curve.real_token_reserves,
        ErrorCodes::InsufficientReserves
    );

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
    require!(
        tokens_out_u64 >= min_tokens_out,
        ErrorCodes::SlippageExceeded
    );

    msg!(
        "initial quote: {} lamports for {} tokens",
        &sol_in,
        &tokens_out,
    );

    /* should have a check to make sure buy does not exceed max per wallet here */

    /* depending on whether or not the user was referred, calculate and send fees */
    if global.fee_basis_points > 0 {
        if ctx.accounts.referrer.key() == global.fee_recipient {
            /* if referrer is default ( fee_recipient ) just send the whole trade_fee at once */
            let trade_fee =
                (sol_in * (global.fee_basis_points - global.ref_share_basis_points)) / 10_000;
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
        } else {
            /* if not then send the fee_recipient's and referrer's rewards separately */
            let fee_recipient_reward =
                (sol_in * (global.fee_basis_points - global.ref_share_basis_points)) / 10_000;
            invoke(
                &system_instruction::transfer(
                    &ctx.accounts.user.key(),
                    &ctx.accounts.fee_recipient.key(),
                    fee_recipient_reward,
                ),
                &[
                    ctx.accounts.user.to_account_info(),
                    ctx.accounts.fee_recipient.to_account_info(),
                    ctx.accounts.system_program.to_account_info(),
                ],
            )?;

            let referrer_reward = (sol_in * global.ref_share_basis_points) / 10_000;
            invoke(
                &system_instruction::transfer(
                    &ctx.accounts.user.key(),
                    &ctx.accounts.referrer.key(),
                    referrer_reward,
                ),
                &[
                    ctx.accounts.user.to_account_info(),
                    ctx.accounts.referrer.to_account_info(),
                    ctx.accounts.system_program.to_account_info(),
                ],
            )?;
        }
    }

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
    bonding_curve.virtual_sol_reserves += sol_in;
    bonding_curve.real_sol_reserves += sol_in;

    if bonding_curve.real_token_reserves == 0 {
        bonding_curve.complete = true;
    }

    Ok(())
}
