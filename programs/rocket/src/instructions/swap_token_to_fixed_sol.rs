use anchor_lang::prelude::*;

use anchor_spl::token::{transfer, Transfer};
use num_bigint::BigInt;
use num_traits::cast::ToPrimitive;

use crate::errors::ErrorCodes;
use crate::instructions::swap::Swap;

pub fn swap_token_to_fixed_sol(ctx: Context<Swap>, sol_out: u64, max_tokens_in: u64) -> Result<()> {
    let bonding_curve = &ctx.accounts.bonding_curve;
    bonding_curve.print()?;

    /* fail if the bonding curve is already complete */
    if bonding_curve.complete {
        return err!(ErrorCodes::BondingCurveComplete);
    }

    /* fail if sol_out is greater than real_sol_reserves */
    if sol_out > bonding_curve.real_sol_reserves {
        return err!(ErrorCodes::InsufficientReserves);
    }

    let sol_out_bigint = BigInt::from(sol_out);

    let virtual_sol_reserves = BigInt::from(bonding_curve.virtual_sol_reserves);
    let virtual_token_reserves = BigInt::from(bonding_curve.virtual_token_reserves);

    let mul_result = &virtual_token_reserves * &virtual_sol_reserves;
    let sub_result = &virtual_sol_reserves - sol_out_bigint;
    let div_result = mul_result / sub_result;
    let tokens_in = &div_result - &virtual_token_reserves;
    let tokens_in_u64 = tokens_in.to_u64().unwrap();

    /* make sure tokens_in_u64 is less than or equal to max_tokens_in */
    if tokens_in_u64 > max_tokens_in {
        return err!(ErrorCodes::SlippageExceeded);
    }

    /* should calculate fees here too */

    msg!(
        "initial quote: {} tokens for {} lamports",
        &tokens_in_u64,
        &sol_out,
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
        tokens_in_u64,
    )?;

    /* transfer sol from bonding curve to user */
    if **ctx
        .accounts
        .bonding_curve
        .to_account_info()
        .try_borrow_lamports()?
        < sol_out
    {
        return err!(ErrorCodes::InsufficientReserves);
    }
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

    /* transfer fees to fee recipient */

    Ok(())
}
