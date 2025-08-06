pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("HpUXGMqxPUyT5msV9Wrm7dpGu6aNCivderuCVJeV3n11");

#[program]
pub mod decentralized_lending_platoform {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        initialize::handler(ctx)
    }

    pub fn initialize_liquidity_pool(
        ctx: Context<InitializeLiquidityPool>, 
        ltv_ratio: u8, 
        liquidation_threshold: u8, 
        liquidation_penalty: u8, 
        interest_rate: u8
    ) -> Result<()> {
        instructions::initialize_liquidity_pool(ctx, ltv_ratio, liquidation_threshold, liquidation_penalty, interest_rate)
    }

    pub fn update_pool_parameters(
        ctx: Context<SetPoolParameters>,
        new_ltv_ratio: u8,
        new_liquidation_threshold: u8,
        new_liquidation_penalty: u8,
        new_interest_rate: u8
    ) -> Result<()> {
        instructions::set_pool_parameters(ctx, new_ltv_ratio, new_liquidation_threshold, new_liquidation_penalty, new_interest_rate)
    }

    pub fn initialize_liquidity_provider(ctx: Context<InitLiquidityProvider>) -> Result<()> {
        instructions::initialize_liquidity_provider(ctx)
    }

    pub fn provide_liquidity(ctx: Context<ProvideLiquidity>, token_a_amount: u64, token_b_amount: u64) -> Result<()> {
        instructions::provide_liquidity(ctx, token_a_amount, token_b_amount)
    }

    pub fn borrow_funds(ctx: Context<BorrowFunds>, amount: u64, borrow_duration: u8) -> Result<()> {
        instructions::borrow_funds(ctx, amount, borrow_duration)
    }

    pub fn repay_funds(ctx: Context<RepayFunds>, repay_amount: u64) -> Result<()> {
        instructions::repay_funds(ctx, repay_amount)
    }
}

/*instructions : 
    init_reserve(mint, initial_liquidity)
    deposit_collateral(amount, mint)
    borrow(amount, mint)
    repay(amount, mint)
    withdraw_collateral(amount, mint)
    liquidate(user, repay_mint)
    accrue_interest()
*/