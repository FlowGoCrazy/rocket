use anchor_lang::prelude::*;

use anchor_spl::{
    associated_token::AssociatedToken,
    metadata::{
        create_metadata_accounts_v3, mpl_token_metadata::types::DataV2, CreateMetadataAccountsV3,
        Metadata as Metaplex,
    },
    token::{mint_to, set_authority, Mint, MintTo, SetAuthority, Token, TokenAccount},
};
use spl_token::instruction::AuthorityType;

use crate::errors::ErrorCodes;
use crate::state::bonding_curve::BondingCurve;
use crate::state::global::Global;

pub fn create(ctx: Context<Create>, params: CreateParams) -> Result<()> {
    let global = &ctx.accounts.global;

    /* fail if global hasnt been initialized */
    if !global.initialized {
        return err!(ErrorCodes::GlobalUninitialized);
    }

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
            seller_fee_basis_points: 0, /* maybe implement later for token creators */
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
        global.token_total_supply,
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
    bonding_curve.virtual_token_reserves = global.initial_virtual_token_reserves;
    bonding_curve.virtual_sol_reserves = global.initial_virtual_sol_reserves;
    bonding_curve.real_token_reserves = global.initial_real_token_reserves;
    bonding_curve.real_sol_reserves = 0;
    bonding_curve.token_total_supply = global.token_total_supply;
    bonding_curve.complete = false;

    Ok(())
}

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
        seeds = [
            b"global",
        ],
        bump,
    )]
    pub global: Account<'info, Global>,

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
