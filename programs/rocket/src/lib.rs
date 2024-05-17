use anchor_lang::prelude::*;
use anchor_lang::solana_program::entrypoint::ProgramResult;
use anchor_spl::{
    associated_token::AssociatedToken,
    metadata::{
        create_metadata_accounts_v3, mpl_token_metadata::types::DataV2, CreateMetadataAccountsV3,
        Metadata as Metaplex,
    },
    token::{mint_to, set_authority, Mint, MintTo, SetAuthority, Token, TokenAccount},
};
use spl_token::instruction::AuthorityType;

declare_id!("8ppDaTFZgYJpPCrpxLow3Bq5HzZicQ6M63MGeXHPoEGb");

#[program]
pub mod rocket {
    use super::*;

    pub fn create(ctx: Context<Create>) -> ProgramResult {
        /* init metadata for new mint */
        create_metadata_accounts_v3(
            CpiContext::new(
                ctx.accounts.token_metadata_program.to_account_info(),
                CreateMetadataAccountsV3 {
                    payer: ctx.accounts.signer.to_account_info(),
                    update_authority: ctx.accounts.mint.to_account_info(),
                    mint: ctx.accounts.mint.to_account_info(),
                    metadata: ctx.accounts.metadata.to_account_info(),
                    mint_authority: ctx.accounts.mint.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info(),
                },
            ),
            DataV2 {
                name: "Test Rocket Token".to_string(),
                symbol: "TRT".to_string(),
                uri: "https://cf-ipfs.com/ipfs/QmSaKVNYHCc4cRU4Wks8nbYqpUr3ZpGdTi7mRdmcrXD9h6"
                    .to_string(),
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
            1_000_000_000,
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
        bonding_curve.virtual_token_reserves = 0;
        bonding_curve.virtual_sol_reserves = 0;
        bonding_curve.real_token_reserves = 1_000_000_000;
        bonding_curve.real_sol_reserves = 0;
        bonding_curve.token_total_supply = 1_000_000_000;
        bonding_curve.complete = false;

        Ok(())
    }

    pub fn buy(ctx: Context<Buy>) -> ProgramResult {
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

        Ok(())
    }
}

/* CONTEXT */
#[derive(Accounts)]
pub struct Create<'info> {
    #[account(
        init,
        payer = signer,
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
        payer = signer,
        space = 8 + BondingCurve::SIZE,
    )]
    pub bonding_curve: Account<'info, BondingCurve>,

    #[account(
        init,
        payer = signer,
        associated_token::mint = mint,
        associated_token::authority = bonding_curve,
    )]
    pub associated_bonding_curve: Account<'info, TokenAccount>,

    /// CHECK: New Metaplex Account
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,

    #[account(mut)]
    pub signer: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub token_metadata_program: Program<'info, Metaplex>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
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

    #[account(mut)]
    pub signer: Signer<'info>,
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
