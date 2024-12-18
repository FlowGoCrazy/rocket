use anchor_lang::prelude::*;

use anchor_lang::solana_program::{program::invoke, system_instruction};
use anchor_spl::token::{transfer, Transfer};
use num_bigint::BigInt;
use num_traits::cast::ToPrimitive;

use crate::errors::ErrorCodes;
use crate::instructions::swap::Swap;

pub fn swap_fixed_token_to_sol(ctx: Context<Swap>, tokens_in: u64, min_sol_out: u64) -> Result<()> {
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

    /* fail if min_sol_out is greater than real_sol_reserves */
    require!(
        min_sol_out <= bonding_curve.real_sol_reserves,
        ErrorCodes::InsufficientReserves
    );

    let tokens_in_bigint = BigInt::from(tokens_in);

    let virtual_sol_reserves = BigInt::from(bonding_curve.virtual_sol_reserves);
    let virtual_token_reserves = BigInt::from(bonding_curve.virtual_token_reserves);

    let mul_result = &virtual_token_reserves * &virtual_sol_reserves;
    let sub_result = &virtual_token_reserves + tokens_in_bigint;
    let div_result = mul_result / sub_result;
    let sol_out = &virtual_sol_reserves - div_result;

    /* set sol out to either the quote or remaining real sol reserves, making sure it cant be higher than the amount remaining */
    let sol_out_u64 = sol_out.to_u64().unwrap();
    let sol_out_u64 = std::cmp::min(sol_out_u64, bonding_curve.real_sol_reserves);

    /* make sure sol_out_u64 is more than min_sol_out */
    require!(sol_out_u64 >= min_sol_out, ErrorCodes::SlippageExceeded);

    msg!(
        "initial quote: {} lamports for {} tokens",
        &sol_out_u64,
        &tokens_in,
    );

    /* depending on whether or not the user was referred, calculate and send fees */
    let referrer_ref = &ctx.accounts.referrer_ref;

    if global.fee_basis_points > 0 {
        if referrer_ref.owner == global.fee_recipient {
            /* if referrer account is owned by default ( fee_recipient ) just send the whole trade_fee directly to fee recipient */
            let trade_fee = (sol_out_u64 * global.fee_basis_points) / 10_000;
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
                (sol_out_u64 * (global.fee_basis_points - global.ref_share_basis_points)) / 10_000;
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

            let referrer_reward = (sol_out_u64 * global.ref_share_basis_points) / 10_000;
            invoke(
                &system_instruction::transfer(
                    &ctx.accounts.user.key(),
                    &ctx.accounts.referrer_ref.key(),
                    referrer_reward,
                ),
                &[
                    ctx.accounts.user.to_account_info(),
                    ctx.accounts.referrer_ref.to_account_info(),
                    ctx.accounts.system_program.to_account_info(),
                ],
            )?;

            /* add reward to referrer's user_ref account balance */
            let referrer_ref = &mut ctx.accounts.referrer_ref;
            referrer_ref.balance += referrer_reward;
        }
    }

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
        tokens_in,
    )?;

    /* transfer sol from bonding curve to user */
    require!(
        **ctx
            .accounts
            .bonding_curve
            .to_account_info()
            .try_borrow_lamports()?
            >= sol_out_u64,
        ErrorCodes::InsufficientReserves
    );
    **ctx
        .accounts
        .bonding_curve
        .to_account_info()
        .try_borrow_mut_lamports()? -= sol_out_u64;
    **ctx
        .accounts
        .user
        .to_account_info()
        .try_borrow_mut_lamports()? += sol_out_u64;

    /* update bonding curve info */
    let bonding_curve = &mut ctx.accounts.bonding_curve;

    msg!(
        "virtual {}, real {}, sol out {}",
        bonding_curve.virtual_sol_reserves,
        bonding_curve.real_sol_reserves,
        sol_out_u64
    );

    bonding_curve.virtual_token_reserves += tokens_in;
    bonding_curve.real_token_reserves += tokens_in;
    bonding_curve.virtual_sol_reserves -= sol_out_u64;
    bonding_curve.real_sol_reserves -= sol_out_u64;

    Ok(())
}
