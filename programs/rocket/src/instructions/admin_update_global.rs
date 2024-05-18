use anchor_lang::prelude::*;

use crate::constants::ADMIN;
use crate::state::global::Global;

pub fn admin_update_global(ctx: Context<UpdateGlobal>, params: UpdateGlobalParams) -> Result<()> {
    let global = &mut ctx.accounts.global;
    global.initialized = true;
    global.fee_recipient = params.fee_recipient;
    global.fee_basis_points = params.fee_basis_points;
    global.ref_share_basis_points = params.ref_share_basis_points;
    global.initial_virtual_token_reserves = params.initial_virtual_token_reserves;
    global.initial_virtual_sol_reserves = params.initial_virtual_sol_reserves;
    global.initial_real_token_reserves = params.initial_real_token_reserves;
    global.token_total_supply = params.token_total_supply;

    Ok(())
}

#[derive(Accounts)]
pub struct UpdateGlobal<'info> {
    #[account(
        init_if_needed, /* safe from re-initialization attack because this is an admin only instruction */
        seeds = [
            b"global",
        ],
        bump,
        payer = admin,
        space = 8 + Global::SIZE,
    )]
    pub global: Account<'info, Global>,

    #[account(mut, address = ADMIN)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct UpdateGlobalParams {
    pub fee_recipient: Pubkey,
    pub fee_basis_points: u64,
    pub ref_share_basis_points: u64,

    pub initial_virtual_token_reserves: u64,
    pub initial_virtual_sol_reserves: u64,
    pub initial_real_token_reserves: u64,
    pub token_total_supply: u64,
}
