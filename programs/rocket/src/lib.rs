use anchor_lang::prelude::*;
use anchor_lang::solana_program::{entrypoint::ProgramResult, program::invoke, system_instruction};
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
use spl_token::instruction::AuthorityType;
use std::cmp;

declare_id!("8ppDaTFZgYJpPCrpxLow3Bq5HzZicQ6M63MGeXHPoEGb");

#[program]
pub mod rocket {
    use super::*;

    pub fn create(ctx: Context<Create>, params: CreateParams) -> ProgramResult {
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

    pub fn buy(ctx: Context<Buy>, sol_in: u64) -> ProgramResult {
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

        let buy_sol_amount = BigInt::from(sol_in);

        let real_token_reserves = BigInt::from(bonding_curve.real_token_reserves);
        let virtual_sol_reserves = BigInt::from(bonding_curve.virtual_sol_reserves);
        let virtual_token_reserves = BigInt::from(bonding_curve.virtual_token_reserves);

        let mul_result = &virtual_sol_reserves * &virtual_token_reserves;
        let add_result = &virtual_sol_reserves + buy_sol_amount;
        let div_result = (mul_result / add_result) + 1;
        let sub_result = &virtual_token_reserves - div_result;
        let tokens_out = cmp::min(&sub_result, &real_token_reserves);
        let (_, tokens_out_vec) = tokens_out.to_u64_digits();
        let tokens_out_u64 = tokens_out_vec[0];

        /* should calculate fees here too */

        msg!(
            "initial quote: {} lamports for {} tokens",
            &sol_in,
            &tokens_out,
            &tokens_out_u64,
        );

        /* should have a check to make sure buy does not exceed max per wallet here */

        /* transfer sol to bonding curve */
        invoke(
            &system_instruction::transfer(
                &ctx.accounts.user.key(),
                &ctx.accounts.bonding_curve.key(),
                sol_in, /* might need changed later */
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
pub struct Buy<'info> {
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
