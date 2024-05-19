use anchor_lang::prelude::*;

use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};

use crate::state::bonding_curve::BondingCurve;
use crate::state::global::Global;
use crate::state::user_ref::UserRef;

#[derive(Accounts)]
pub struct Swap<'info> {
    #[account(
        seeds = [
            b"global",
        ],
        bump,
    )]
    pub global: Account<'info, Global>,

    /// CHECK: Checked Within Instruction
    #[account(mut)]
    pub fee_recipient: AccountInfo<'info>,

    /// CHECK: Checked Within Instruction
    #[account(mut)]
    pub referrer: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [
            referrer.key().as_ref(),
            b"ref",
        ],
        bump,
    )]
    pub referrer_ref: Account<'info, UserRef>,

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
        mut,
        associated_token::mint = mint,
        associated_token::authority = bonding_curve,
    )]
    pub associated_bonding_curve: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = user,
    )]
    pub associated_user: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}
