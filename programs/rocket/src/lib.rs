use anchor_lang::prelude::*;

pub mod constants;
pub mod errors;
pub mod instructions;
pub mod state;
use instructions::*;

declare_id!("8ppDaTFZgYJpPCrpxLow3Bq5HzZicQ6M63MGeXHPoEGb");

#[program]
pub mod rocket {
    use super::*;

    // allow the admin to update global state
    pub fn admin_update_global(
        ctx: Context<UpdateGlobal>,
        params: UpdateGlobalParams,
    ) -> Result<()> {
        admin_update_global::admin_update_global(ctx, params)
    }

    // allow the admin to withdraw liquidity for migration
    pub fn admin_withdraw(ctx: Context<AdminWithdraw>) -> Result<()> {
        admin_withdraw::admin_withdraw(ctx)
    }

    // allow a signer to initialize a user_ref account for themself or another user
    pub fn init_user_ref(ctx: Context<InitUserRef>) -> Result<()> {
        init_user_ref::init_user_ref(ctx)
    }

    /// allow a user to create a new token and initialize a bonding curve
    pub fn create(ctx: Context<Create>, params: CreateParams) -> Result<()> {
        create::create(ctx, params)
    }

    /// allow buyers to swap a fixed amount of sol for a variable amount of tokens
    pub fn swap_fixed_sol_to_token(
        ctx: Context<Swap>,
        sol_in: u64,
        min_tokens_out: u64,
    ) -> Result<()> {
        swap_fixed_sol_to_token::swap_fixed_sol_to_token(ctx, sol_in, min_tokens_out)
    }

    /// allow buyers to swap a variable amount of sol for a fixed amount of tokens
    pub fn swap_sol_to_fixed_token(
        ctx: Context<Swap>,
        tokens_out: u64,
        max_sol_in: u64,
    ) -> Result<()> {
        swap_sol_to_fixed_token::swap_sol_to_fixed_token(ctx, tokens_out, max_sol_in)
    }

    /// allow sellers to swap a fixed amount of tokens for a variable amount of sol
    pub fn swap_fixed_token_to_sol(
        ctx: Context<Swap>,
        tokens_in: u64,
        min_sol_out: u64,
    ) -> Result<()> {
        swap_fixed_token_to_sol::swap_fixed_token_to_sol(ctx, tokens_in, min_sol_out)
    }

    /// allow sellers to swap a variable amount of tokens for a fixed amount of sol
    pub fn swap_token_to_fixed_sol(
        ctx: Context<Swap>,
        sol_out: u64,
        max_tokens_in: u64,
    ) -> Result<()> {
        swap_token_to_fixed_sol::swap_token_to_fixed_sol(ctx, sol_out, max_tokens_in)
    }
}
