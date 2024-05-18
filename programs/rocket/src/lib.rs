use anchor_lang::prelude::*;
use anchor_lang::solana_program::{program::invoke, system_instruction};
use anchor_spl::{
    associated_token::AssociatedToken,
    metadata::{
        create_metadata_accounts_v3, mpl_token_metadata::types::DataV2, CreateMetadataAccountsV3,
        Metadata as Metaplex,
    },
    token::{
        mint_to, set_authority, transfer, Mint, MintTo, SetAuthority, Token, TokenAccount, Transfer,
    },
};
use num_bigint::BigInt;
use num_traits::cast::ToPrimitive;
use spl_token::instruction::AuthorityType;
use std::cmp;

declare_id!("8ppDaTFZgYJpPCrpxLow3Bq5HzZicQ6M63MGeXHPoEGb");

#[program]
pub mod rocket {
    use super::*;

    /// allow a user to create a new token and initialize a bonding curve
    pub fn create(ctx: Context<Create>, params: CreateParams) -> Result<()> {
        /* init metadata for new mint */
        create_metadata_accounts_v3(
            CpiContext::new(
                ctx.accounts.token_metadata_program.to_account_info(),
                CreateMetadataAccountsV3 {
                    payer: ctx.accounts.user.to_account_info(),
                    update_authority: ctx.accounts.mint.to_account_info(),
                    mint: ctx.accounts.mint.to_account_info(),
                    metadata: ctx.accounts.metadata.to_account_info(),
                    mint_authority: ctx.accounts.mint.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info(),
                },
            ),
            DataV2 {
                name: params.name,
                symbol: params.symbol,
                uri: params.uri,
                seller_fee_basis_points: 0,
                creators: None,
                collection: None,
                uses: None,
            },
            false,
            true,
            None,
        )?;

        /* mint tokens to associated bonding curve */
        mint_to(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.mint.to_account_info(),
                    to: ctx.accounts.associated_bonding_curve.to_account_info(),
                    authority: ctx.accounts.mint.to_account_info(),
                },
            ),
            1_000_000_000_000_000,
        )?;

        /* revoke mint authority */
        set_authority(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                SetAuthority {
                    account_or_mint: ctx.accounts.mint.to_account_info(),
                    current_authority: ctx.accounts.mint.to_account_info(),
                },
            ),
            AuthorityType::MintTokens,
            None,
        )?;

        /* set initial bonding curve state */
        let bonding_curve = &mut ctx.accounts.bonding_curve;
        bonding_curve.virtual_token_reserves = 1_073_000_000_000_000; /* always starts at this number */
        bonding_curve.virtual_sol_reserves = 30_000_000_000; /* always starts at this number */
        bonding_curve.real_token_reserves = 793_100_000_000_000; /* always starts at this number */
        bonding_curve.real_sol_reserves = 0;
        bonding_curve.token_total_supply = 1_000_000_000_000_000; /* 1 billion + 6 decimals */
        bonding_curve.complete = false;

        Ok(())
    }

    /// allow buyers to swap a fixed amount of sol for a variable amount of tokens
    pub fn swap_fixed_sol_to_token(
        ctx: Context<Swap>,
        sol_in: u64,
        min_tokens_out: u64,
    ) -> Result<()> {
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
            return err!(CustomError::BondingCurveComplete);
        }

        /* fail if min_tokens_out is greater than real_token_reserves */
        if min_tokens_out > bonding_curve.real_token_reserves {
            return err!(CustomError::InsufficientReserves);
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
        let tokens_out_u64 = tokens_out.to_u64().unwrap();

        /* make sure tokens_out_u64 is more than min_tokens_out */
        if tokens_out_u64 < min_tokens_out {
            return err!(CustomError::SlippageExceeded);
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

    /// allow buyers to swap a variable amount of sol for a fixed amount of tokens
    pub fn swap_sol_to_fixed_token(
        ctx: Context<Swap>,
        tokens_out: u64,
        max_sol_in: u64,
    ) -> Result<()> {
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
            return err!(CustomError::BondingCurveComplete);
        }

        /* fail if tokens_out is greater than real_token_reserves */
        if tokens_out > bonding_curve.real_token_reserves {
            return err!(CustomError::InsufficientReserves);
        }

        let tokens_out_bigint = BigInt::from(tokens_out);

        let virtual_sol_reserves = BigInt::from(bonding_curve.virtual_sol_reserves);
        let virtual_token_reserves = BigInt::from(bonding_curve.virtual_token_reserves);

        let mul_result = &virtual_token_reserves * &virtual_sol_reserves;
        let sub_result = &virtual_token_reserves - tokens_out_bigint;
        let div_result = mul_result / sub_result;
        let sol_in = div_result - &virtual_sol_reserves;
        let sol_in_u64 = sol_in.to_u64().unwrap();

        /* make sure sol_in_u64 is less than max_sol_in */
        if sol_in_u64 > max_sol_in {
            return err!(CustomError::SlippageExceeded);
        }

        /* should calculate fees here too */

        msg!(
            "initial quote: {} tokens for {} lamports",
            &tokens_out,
            &sol_in_u64,
        );

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

    /// allow sellers to swap a fixed amount of tokens for a variable amount of sol
    pub fn swap_fixed_token_to_sol(
        ctx: Context<Swap>,
        tokens_in: u64,
        min_sol_out: u64,
    ) -> Result<()> {
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
            return err!(CustomError::BondingCurveComplete);
        }

        /* fail if min_sol_out is greater than real_sol_reserves */
        if min_sol_out > bonding_curve.real_sol_reserves {
            return err!(CustomError::InsufficientReserves);
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
            return err!(CustomError::SlippageExceeded);
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
            return err!(CustomError::InsufficientReserves);
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

    /// allow sellers to swap a variable amount of tokens for a fixed amount of sol
    pub fn swap_token_to_fixed_sol(
        ctx: Context<Swap>,
        sol_out: u64,
        max_tokens_in: u64,
    ) -> Result<()> {
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
            return err!(CustomError::BondingCurveComplete);
        }

        /* fail if sol_out is greater than real_sol_reserves */
        if sol_out > bonding_curve.real_sol_reserves {
            return err!(CustomError::InsufficientReserves);
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
            return err!(CustomError::SlippageExceeded);
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
            return err!(CustomError::InsufficientReserves);
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
}

/* CONTEXT */
#[derive(Accounts)]
pub struct Create<'info> {
    #[account(
        init,
        payer = user,
        mint::decimals = 6,
        mint::authority = mint,
    )]
    pub mint: Account<'info, Mint>,

    #[account(
        init,
        seeds = [
            mint.key().as_ref(),
            b"bonding_curve",
        ],
        bump,
        payer = user,
        space = 8 + BondingCurve::SIZE,
    )]
    pub bonding_curve: Account<'info, BondingCurve>,

    #[account(
        init,
        payer = user,
        associated_token::mint = mint,
        associated_token::authority = bonding_curve,
    )]
    pub associated_bonding_curve: Account<'info, TokenAccount>,

    /// CHECK: New Metaplex Account
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub token_metadata_program: Program<'info, Metaplex>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct CreateParams {
    pub name: String,
    pub symbol: String,
    pub uri: String,
}

#[derive(Accounts)]
pub struct Swap<'info> {
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
        associated_token::mint = mint,
        associated_token::authority = bonding_curve,
    )]
    pub associated_bonding_curve: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        associated_token::mint = mint,
        associated_token::authority = user,
    )]
    pub associated_user: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
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

/* ERRORS */
#[error_code]
pub enum CustomError {
    #[msg("slippage exceeded: output less than minimum required")]
    SlippageExceeded,

    #[msg("bonding curve complete: trading locked until migration to raydium")]
    BondingCurveComplete,

    #[msg("insufficient funds: not enough funds to complete transaction")]
    InsufficientFunds,

    #[msg("insufficient reserves: not enough funds in reserve to complete transaction")]
    InsufficientReserves,
}
