use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::LiquidityPool;
use crate::error::ErrorCode;

#[derive(Accounts)]
pub struct InitializeLiquidityPool<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,

    pub token_mint_a: Account<'info, Mint>,

    pub token_mint_b: Account<'info, Mint>,

    #[account(
        init,
        payer = creator,
        space = 8 + LiquidityPool::INIT_SPACE,
        seeds = [b"liquidity_pool", token_mint_a.key().as_ref(), token_mint_b.key().as_ref(), creator.key().as_ref()],
        bump
    )]
    pub liquidity_pool: Account<'info, LiquidityPool>,

    #[account(
        init,
        payer = creator,
        seeds = [b"lp_token_mint", liquidity_pool.key().as_ref()],
        bump,
        mint::authority = liquidity_pool,
        mint::decimals = 6
    )]
    pub lp_token_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = creator,
        seeds = [b"token_vault_a", token_mint_a.key().as_ref(), liquidity_pool.key().as_ref()],
        bump,
        token::mint = token_mint_a,
        token::authority = liquidity_pool
    )]
    pub token_vault_a: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = creator,
        seeds = [b"token_vault_b", token_mint_b.key().as_ref(), liquidity_pool.key().as_ref()],
        bump,
        token::mint = token_mint_b,
        token::authority = liquidity_pool
    )]
    pub token_vault_b: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = creator,
        seeds = [b"fee_vault_a", token_mint_a.key().as_ref(), liquidity_pool.key().as_ref()],
        bump,
        token::mint = token_mint_a,
        token::authority = liquidity_pool
    )]
    pub fee_vault_a: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = creator,
        seeds = [b"fee_vault_b", token_mint_b.key().as_ref(), liquidity_pool.key().as_ref()],
        bump,
        token::mint = token_mint_b,
        token::authority = liquidity_pool
    )]
    pub fee_vault_b: Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct SetPoolParameters<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,

    #[account(
        mut,
        seeds = [b"liquidity_pool", liquidity_pool.mint_a.key().as_ref(), liquidity_pool.mint_b.key().as_ref(), creator.key().as_ref()],
        bump
    )]
    pub liquidity_pool: Account<'info, LiquidityPool>,
}

pub fn initialize_liquidity_pool(ctx: Context<InitializeLiquidityPool>, ltv_ratio: u8, liquidation_threshold: u8, liquidation_penalty: u8, interest_rate: u8) -> Result<()> {

    require_keys_neq!(ctx.accounts.token_mint_a.key(), ctx.accounts.token_mint_b.key(), ErrorCode::SameTokenMints);

    let liquidity_pool_account = &mut ctx.accounts.liquidity_pool;

    if ltv_ratio == 0 || ltv_ratio > 100 {
        return Err(ErrorCode::InvalidLtv.into());
    }

    if liquidation_threshold == 0 || liquidation_threshold > 100 {
        return Err(ErrorCode::InvalidLiquidationThreshold.into());
    }

    if liquidation_penalty == 0 || liquidation_penalty > 100 {
        return Err(ErrorCode::InvalidLiquidationPanelty.into());
    }

    liquidity_pool_account.set_inner(LiquidityPool {
        authority: ctx.accounts.creator.key(),
        mint_a: ctx.accounts.token_mint_a.key(),
        mint_b: ctx.accounts.token_mint_b.key(),
        lp_mint: ctx.accounts.lp_token_mint.key(),
        vault_a: ctx.accounts.token_vault_a.key(),
        vault_b: ctx.accounts.token_vault_b.key(),
        fees_vault_a: ctx.accounts.fee_vault_a.key(),
        fees_vault_b: ctx.accounts.fee_vault_b.key(),
        total_liquidity: 0,
        total_borrowed_a: 0,
        total_borrowed_b: 0,
        total_borrowed: 0,
        ltv_ratio,
        liquidation_threshold,
        liquidation_penalty,
        interest_rate,
        created_at: Clock::get()?.unix_timestamp,
        lp_supply: 0,
        bump: ctx.bumps.liquidity_pool,
        vault_a_bump: ctx.bumps.token_vault_a,
        vault_b_bump: ctx.bumps.token_vault_b,
        fees_vault_a_bump: ctx.bumps.fee_vault_a,
        fees_vault_b_bump: ctx.bumps.fee_vault_b
    });

    msg!("Liquidity pool initialized successfully!");
    msg!("Liquidity Pool Info:");
    msg!("{:?}", liquidity_pool_account);

    Ok(())
}

pub fn set_pool_parameters(
    ctx: Context<SetPoolParameters>, 
    new_ltv_ratio: u8,
    new_liquidation_threshold: u8,
    new_liquidation_penalty: u8,
    new_interest_rate: u8,
) -> Result<()> {

    require_eq!(ctx.accounts.creator.key(), ctx.accounts.liquidity_pool.authority, ErrorCode::InvalidAuthority);
    require!(new_ltv_ratio <= new_liquidation_threshold, ErrorCode::InvalidLtvThreshold);
    require!(new_liquidation_penalty < 100, ErrorCode::InvalidPenalty);
    require!(new_interest_rate <= 100, ErrorCode::InvalidInterestRate);

    let pool = &mut ctx.accounts.liquidity_pool;
    pool.ltv_ratio = new_ltv_ratio;
    pool.liquidation_threshold = new_liquidation_threshold;
    pool.liquidation_penalty = new_liquidation_penalty;
    pool.interest_rate = new_interest_rate;

    msg!("Updated pool info: {:?}", ctx.accounts.liquidity_pool);

    Ok(())
}
