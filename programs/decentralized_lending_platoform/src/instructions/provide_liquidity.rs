use anchor_lang::prelude::*;
use anchor_spl::token::{mint_to, transfer_checked, MintTo, TransferChecked};
use anchor_spl::{token::Mint, token::TokenAccount, token::Token};
use anchor_spl::associated_token::*;

use crate::{LiquidityPool, LiquidityProviderInfo};

use crate::error::ErrorCode;

#[derive(Accounts)]
pub struct ProvideLiquidity<'info,> {
    #[account(mut)]
    pub provider: Signer<'info>,

    pub token_mint_a: Account<'info, Mint>,

    pub token_mint_b: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [b"lp_token_mint", liquidity_pool.key().as_ref()],
        bump,
        mint::authority = liquidity_pool,
        mint::decimals = 6
    )]
    pub lp_token_mint: Account<'info, Mint>,

     #[account(
        mut,
        seeds = [b"liquidity_pool", token_mint_a.key().as_ref(), token_mint_b.key().as_ref(), liquidity_pool.authority.key().as_ref()],
        bump
    )]
    pub liquidity_pool: Account<'info, LiquidityPool>,

     #[account(
        mut,
        seeds = [b"liquidity_provider", provider.key().as_ref()],
        bump
    )]
    pub liquidity_provider_account: Account<'info, LiquidityProviderInfo>,

    #[account(
        mut,
        associated_token::mint = token_mint_a,
        associated_token::authority = provider
    )]
    pub provider_token_a_ata: Account<'info, TokenAccount>,

     #[account(
        mut,
        associated_token::mint = token_mint_b,
        associated_token::authority = provider
    )]
    pub provider_token_b_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"token_vault_a", token_mint_a.key().as_ref(), liquidity_pool.key().as_ref()],
        bump,
        token::mint = token_mint_a,
        token::authority = liquidity_pool
    )]
    pub token_vault_a: Account<'info, TokenAccount>,

     #[account(
        mut,
        seeds = [b"token_vault_b", token_mint_b.key().as_ref(), liquidity_pool.key().as_ref()],
        bump,
        token::mint = token_mint_b,
        token::authority = liquidity_pool
    )]
    pub token_vault_b: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = lp_token_mint,
        associated_token::authority = provider
    )]
    pub provider_lp_mint_ata: Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,

    pub token_program: Program<'info, Token>,

    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub fn provide_liquidity(ctx: Context<ProvideLiquidity>, token_a_amount: u64, token_b_amount: u64) -> Result<()> {

    require!(token_a_amount == token_b_amount, ErrorCode::InvalidLiquidityAmount);

    let liquidity_pool_account = &mut ctx.accounts.liquidity_pool;
    let liquidity_provider_account = &mut ctx.accounts.liquidity_provider_account;

    let cpi_accounts_1 = TransferChecked {
        from: ctx.accounts.provider_token_a_ata.to_account_info(),
        to: ctx.accounts.token_vault_a.to_account_info(),
        authority: ctx.accounts.provider.to_account_info(),
        mint: ctx.accounts.token_mint_a.to_account_info(),
    };

    let cpi_context1 = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts_1);

    transfer_checked(cpi_context1, token_a_amount, ctx.accounts.token_mint_a.decimals)?;

    let cpi_accounts_2 = TransferChecked {
        from: ctx.accounts.provider_token_b_ata.to_account_info(),
        to: ctx.accounts.token_vault_b.to_account_info(),
        authority: ctx.accounts.provider.to_account_info(),
        mint: ctx.accounts.token_mint_b.to_account_info()
    };

    let cpi_context_2 = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts_2);

    transfer_checked(cpi_context_2, token_b_amount, ctx.accounts.token_mint_b.decimals)?;

    let lp_tokens_to_mint = token_a_amount + token_b_amount;

    let token_mint_a = ctx.accounts.token_mint_a.key();
    let token_mint_b = ctx.accounts.token_mint_b.key();

   let liquidity_pool_seeds: &[&[&[u8]]] = &[&[
        b"liquidity_pool",
        token_mint_a.as_ref(),
        token_mint_b.as_ref(),
        liquidity_pool_account.authority.as_ref(),
        &[ctx.bumps.liquidity_pool],
    ]];

    let mint_to_cpi = MintTo {
        mint: ctx.accounts.lp_token_mint.to_account_info(),
        to: ctx.accounts.provider_lp_mint_ata.to_account_info(),
        authority: liquidity_pool_account.to_account_info()
    };

    let mint_context = CpiContext::new_with_signer(ctx.accounts.token_program.to_account_info(), mint_to_cpi, liquidity_pool_seeds);

    mint_to(mint_context, lp_tokens_to_mint)?;

    liquidity_pool_account.total_liquidity = liquidity_pool_account
        .total_liquidity
        .checked_add(token_a_amount + token_b_amount)
        .ok_or(ErrorCode::Overflow)?;

    liquidity_provider_account.liquidity_pool = liquidity_pool_account.key();

    liquidity_provider_account.provided_token_a = liquidity_provider_account
        .provided_token_a
        .checked_add(token_a_amount)
        .ok_or(ErrorCode::Overflow)?;

    liquidity_provider_account.provided_token_b = liquidity_provider_account
        .provided_token_b
        .checked_add(token_b_amount)
        .ok_or(ErrorCode::Overflow)?;

    liquidity_provider_account.total_liquidity_provided = liquidity_provider_account
        .total_liquidity_provided
        .checked_add(token_a_amount + token_b_amount)
        .ok_or(ErrorCode::Overflow)?;

    liquidity_provider_account.total_lp_tokens = liquidity_provider_account
        .total_lp_tokens
        .checked_add(lp_tokens_to_mint)
        .ok_or(ErrorCode::Overflow)?;

    msg!("Liquidity provided successfully!");
    msg!("Liquidity Pool Account: {:?}", liquidity_pool_account);
    msg!("Liquidity Provider Account: {:?}", liquidity_provider_account);

    Ok(())
}


