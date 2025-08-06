use anchor_lang::prelude::*;

#[account]
#[derive(Debug, InitSpace)]
pub struct LiquidityProviderInfo {
    pub provider: Pubkey,
    pub liquidity_pool: Pubkey,
    pub provided_token_a: u64,
    pub provided_token_b: u64,
    pub total_liquidity_provided: u64, 
    pub total_lp_tokens: u64
}

