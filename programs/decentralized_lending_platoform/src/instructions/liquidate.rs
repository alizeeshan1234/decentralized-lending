use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TransferChecked, Mint, TokenAccount};
use crate::state::{BorrowInfo, LiquidityPool};
use crate::error::ErrorCode;
use crate::BorrowDuration;
use anchor_spl::associated_token::AssociatedToken;

#[derive(Accounts)]
pub struct Liquidate<'info> {
    #[account(mut)]
    pub liquidator: Signer<'info>,

    pub borrower: AccountInfo<'info>,

    pub loan_mint: Account<'info, Mint>,

    pub collateral_mint: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [b"borrower_account", borrower.key().as_ref()],
        bump,
        constraint = borrower_account_info.borrowed_from_pool == liquidity_pool.key()
    )]
    pub borrower_account_info: Account<'info, BorrowInfo>,

    #[account(
        mut,
        seeds = [
            b"liquidity_pool",
            liquidity_pool.mint_a.key().as_ref(),
            liquidity_pool.mint_b.key().as_ref(),
            liquidity_pool.authority.key().as_ref(),
        ],
        bump
    )]
    pub liquidity_pool: Account<'info, LiquidityPool>,

   #[account(
        mut,
        seeds = [b"token_vault_a", liquidity_pool.mint_a.key().as_ref(), liquidity_pool.key().as_ref()],
        bump,
        token::mint = liquidity_pool.mint_a,
        token::authority = liquidity_pool,
    )]
    pub token_vault_a: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"token_vault_b", liquidity_pool.mint_b.key().as_ref(), liquidity_pool.key().as_ref()],
        bump,
        token::mint = liquidity_pool.mint_b,
        token::authority = liquidity_pool,
    )]
    pub token_vault_b: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"fee_vault_a", liquidity_pool.mint_a.key().as_ref(), liquidity_pool.key().as_ref()],
        bump,
        token::mint = liquidity_pool.mint_a,
        token::authority = liquidity_pool,
    )]
    pub fee_vault_a: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"fee_vault_b", liquidity_pool.mint_b.key().as_ref(), liquidity_pool.key().as_ref()],
        bump,
        token::mint = liquidity_pool.mint_b,
        token::authority = liquidity_pool,
    )]
    pub fee_vault_b: Account<'info, TokenAccount>,

    // Borrower's token accounts
    #[account(mut)]
    pub borrower_collateral_ata: Account<'info, TokenAccount>,

    // Liquidator's token account to receive seized collateral
    #[account(
        mut,
        associated_token::mint = collateral_mint,
        associated_token::authority = liquidator,
    )]
    pub liquidator_collateral_ata: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(ctx: Context<Liquidate>) -> Result<()> {
    let clock = Clock::get()?;
    let borrower_info = &ctx.accounts.borrower_account_info;

    // Validate that the loan has expired

    let borrow_duration = borrower_info.borrow_duration;

    let borrow_duration_u8 = match borrow_duration {
        BorrowDuration::TenDays => 10,
        BorrowDuration::TwentyDays => 20,
        BorrowDuration::ThirtyDays => 30,
        _ => 10
    };

    let expiry_time = borrower_info.borrowed_at + borrow_duration_u8;
    require!(
        clock.unix_timestamp > expiry_time,
        ErrorCode::LoanNotExpired
    );
    
    require_keys_eq!(borrower_info.borrower, ctx.accounts.borrower.key(), ErrorCode::InvalidBorrower);

    let amount = borrower_info.total_collateral;

    let cpi_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        TransferChecked {
            from: ctx.accounts.borrower_collateral_ata.to_account_info(),
            to: ctx.accounts.liquidator_collateral_ata.to_account_info(),
            authority: ctx.accounts.borrower.to_account_info(),
            mint: ctx.accounts.collateral_mint.to_account_info(),
        },
    );

    token::transfer_checked(cpi_ctx, amount, ctx.accounts.collateral_mint.decimals)?;

    // Reset borrow info
    let mut borrower_info = &mut ctx.accounts.borrower_account_info;
    borrower_info.total_borrowed = 0;
    borrower_info.total_collateral = 0;
    borrower_info.borrowed_at = 0;

    Ok(())
}
