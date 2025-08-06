use anchor_lang::prelude::*;
use anchor_spl::token::{transfer_checked, Mint, Token, TokenAccount, TransferChecked};
use anchor_spl::associated_token::AssociatedToken;

use crate::{BorrowDuration, BorrowInfo, LiquidityPool};
use crate::error::ErrorCode;


#[derive(Accounts)]
pub struct RepayFunds<'info> {
    #[account(mut)]
    pub borrower: Signer<'info>,

    pub wanted_mint: Account<'info, Mint>,
    pub giving_mint: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [b"borrower_account", borrower.key().as_ref()],
        bump
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

    #[account(
        mut,
        associated_token::mint = wanted_mint,
        associated_token::authority = borrower,
    )]
    pub borrower_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = giving_mint,
        associated_token::authority = borrower,
    )]
    pub borrower_collateral_ata: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn repay_funds(ctx: Context<RepayFunds>, repay_amount: u64) -> Result<()> {
    let borrow_info = &mut ctx.accounts.borrower_account_info;

    require!(
        repay_amount <= borrow_info.total_borrowed,
        ErrorCode::InvalidRepayAmount
    );

    // Determine repayment vault and mint
    let (repay_vault, repay_mint, repay_mint_info, repay_decimals) =
        if borrow_info.borrowed_from_pool == ctx.accounts.liquidity_pool.mint_a {
            (
                &ctx.accounts.token_vault_a,
                &ctx.accounts.wanted_mint,
                &ctx.accounts.wanted_mint,
                ctx.accounts.wanted_mint.decimals,
            )
        } else {
            (
                &ctx.accounts.token_vault_b,
                &ctx.accounts.wanted_mint,
                &ctx.accounts.wanted_mint,
                ctx.accounts.wanted_mint.decimals,
            )
        };

    // Transfer repayment to the vault
    let cpi_accounts = TransferChecked {
        from: ctx.accounts.borrower_ata.to_account_info(),
        to: repay_vault.to_account_info(),
        authority: ctx.accounts.borrower.to_account_info(),
        mint: repay_mint_info.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
    transfer_checked(cpi_ctx, repay_amount, repay_decimals)?;

    borrow_info.total_borrowed -= repay_amount;

    // If loan fully repaid, return collateral
    if borrow_info.total_borrowed == 0 {
        let (collateral_vault, collateral_mint, collateral_decimals) =
            if borrow_info.borrowed_from_pool == ctx.accounts.liquidity_pool.mint_a {
                (
                    &ctx.accounts.fee_vault_b,
                    &ctx.accounts.giving_mint,
                    ctx.accounts.giving_mint.decimals,
                )
            } else {
                (
                    &ctx.accounts.fee_vault_a,
                    &ctx.accounts.giving_mint,
                    ctx.accounts.giving_mint.decimals,
                )
            };

        let cpi_accounts = TransferChecked {
            from: collateral_vault.to_account_info(),
            to: ctx.accounts.borrower_collateral_ata.to_account_info(),
            authority: ctx.accounts.liquidity_pool.to_account_info(),
            mint: collateral_mint.to_account_info(),
        };

        let signer_seeds: &[&[&[u8]]] = &[
            &[
                b"liquidity_pool",
                ctx.accounts.liquidity_pool.mint_a.as_ref(),
                ctx.accounts.liquidity_pool.mint_b.as_ref(),
                ctx.accounts.liquidity_pool.authority.as_ref(),
                &[ctx.bumps.liquidity_pool],
            ]
        ];

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            cpi_accounts,
            signer_seeds,
        );
        transfer_checked(
            cpi_ctx,
            borrow_info.total_collateral,
            collateral_decimals,
        )?;

        borrow_info.total_collateral = 0;
    }

    Ok(())
}
