use anchor_lang::prelude::*;

use crate::state::user_ref::UserRef;

pub fn init_user_ref(_ctx: Context<InitUserRef>) -> Result<()> {
    Ok(())
}

#[derive(Accounts)]
pub struct InitUserRef<'info> {
    /// CHECK: Only Used To Find PDA
    #[account(mut)]
    pub user: UncheckedAccount<'info>,

    #[account(
        init,
        seeds = [
            user.key().as_ref(),
            b"ref",
        ],
        bump,
        payer = signer,
        space = 8 + UserRef::SIZE,
    )]
    pub user_ref: Account<'info, UserRef>,

    #[account(mut)]
    pub signer: Signer<'info>,

    pub system_program: Program<'info, System>,
}
