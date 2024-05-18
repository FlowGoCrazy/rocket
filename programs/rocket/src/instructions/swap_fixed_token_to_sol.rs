use anchor_lang::prelude::*;

use anchor_spl::token::{transfer, Transfer};
use num_bigint::BigInt;
use num_traits::cast::ToPrimitive;

use crate::errors::ErrorCodes;
use crate::instructions::swap::Swap;

pub fn swap_fixed_token_to_sol(ctx: Context<Swap>, tokens_in: u64, min_sol_out: u64) -> Result<()> {
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

    /* fail if the bonding curve is already complete */
    if bonding_curve.complete {
        return err!(ErrorCodes::BondingCurveComplete);
    }

    /* fail if min_sol_out is greater than real_sol_reserves */
    if min_sol_out > bonding_curve.real_sol_reserves {
        return err!(ErrorCodes::InsufficientReserves);
    }

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
    if sol_out_u64 < min_sol_out {
        return err!(ErrorCodes::SlippageExceeded);
    }

    /* should calculate fees here too */

    msg!(
        "initial quote: {} lamports for {} tokens",
        &sol_out_u64,
        &tokens_in,
    );

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
    if **ctx
        .accounts
        .bonding_curve
        .to_account_info()
        .try_borrow_lamports()?
        < sol_out_u64
    {
        return err!(ErrorCodes::InsufficientReserves);
    }
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

    /* transfer fees to fee recipient */

    Ok(())
}
