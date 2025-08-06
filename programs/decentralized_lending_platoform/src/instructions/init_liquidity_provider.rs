use anchor_lang::prelude::*;
use crate::state::LiquidityProviderInfo;

#[derive(Accounts)]
pub struct InitLiquidityProvider<'info> {
    #[account(mut)]
    pub provider: Signer<'info>,

    #[account(
        init,
        payer = provider,
        space = 8 + LiquidityProviderInfo::INIT_SPACE,
        seeds = [b"liquidity_provider", provider.key().as_ref()],
        bump
    )]
    pub liquidity_provider_account: Account<'info, LiquidityProviderInfo>,

    pub system_program: Program<'info, System>,
}

pub fn initialize_liquidity_provider(ctx: Context<InitLiquidityProvider>) -> Result<()> {

    let liquidity_provider_account = &mut ctx.accounts.liquidity_provider_account;

    liquidity_provider_account.set_inner(LiquidityProviderInfo {
        provider: ctx.accounts.provider.key(),
        liquidity_pool: Pubkey::default(),
        provided_token_a: 0,
        provided_token_b: 0,
        total_liquidity_provided: 0,
        total_lp_tokens: 0
    });

    msg!("Initialized liquidity provider: {:?}", liquidity_provider_account);

    Ok(())
}