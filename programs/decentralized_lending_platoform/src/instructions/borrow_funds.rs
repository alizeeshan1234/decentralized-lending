use anchor_lang::prelude::*;
use anchor_spl::token::{transfer_checked, Mint, Token, TokenAccount, TransferChecked};
use anchor_spl::associated_token::AssociatedToken;

use crate::{BorrowDuration, BorrowInfo, LiquidityPool};
use crate::error::ErrorCode;

#[derive(Accounts)]
pub struct BorrowFunds<'info> {
    #[account(mut)]
    pub borrower: Signer<'info>,

    pub wanted_mint: Account<'info, Mint>,
    pub giving_mint: Account<'info, Mint>,

    #[account(
        init_if_needed,
        payer = borrower,
        space = 8 + BorrowInfo::INIT_SPACE,
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


pub fn borrow_funds(
    ctx: Context<BorrowFunds>,
    amount: u64,
    borrow_duration: u8,
) -> Result<()> {
    let borrower = &ctx.accounts.borrower;
    let liquidity_pool = &mut ctx.accounts.liquidity_pool;
    let giving_mint = &ctx.accounts.giving_mint;
    let wanted_mint = &ctx.accounts.wanted_mint;
    let borrower_account_info = &mut ctx.accounts.borrower_account_info;

    // Step 2: Determine which token is collateral and which is being borrowed
    let (collateral_mint, borrow_mint) = if giving_mint.key() == liquidity_pool.mint_a {
        (liquidity_pool.mint_a, liquidity_pool.mint_b)
    } else if giving_mint.key() == liquidity_pool.mint_b {
        (liquidity_pool.mint_b, liquidity_pool.mint_a)
    } else {
        return Err(ErrorCode::InvalidMint.into());
    };

    // Step 3: Calculate amount to borrow using LTV ratio (assume 50%) d d
    let ltv = liquidity_pool.ltv_ratio; // e.g. 50 means 50%
    let borrow_amount = amount
        .checked_mul(ltv as u64)
        .ok_or(ErrorCode::MathOverflow)?
        .checked_div(100)
        .ok_or(ErrorCode::MathOverflow)?;

    // Step 4: Transfer collateral from borrower to vault
    transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.borrower_collateral_ata.to_account_info(),
                to: if collateral_mint == liquidity_pool.mint_a {
                    ctx.accounts.token_vault_a.to_account_info()
                } else {
                    ctx.accounts.token_vault_b.to_account_info()
                },
                authority: borrower.to_account_info(),
                mint: giving_mint.to_account_info(),
            },
        ),
        amount,
        giving_mint.decimals,
    )?;

    // Step 5: Transfer borrowed tokens from vault to borrower
    let vault_account = if borrow_mint == liquidity_pool.mint_a {
        ctx.accounts.token_vault_a.to_account_info()
    } else {
        ctx.accounts.token_vault_b.to_account_info()
    };

    let seeds = &[
        b"liquidity_pool",
        liquidity_pool.mint_a.as_ref(),
        liquidity_pool.mint_b.as_ref(),
        liquidity_pool.authority.as_ref(),
        &[ctx.bumps.liquidity_pool]
    ];

    transfer_checked(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: vault_account.clone(),
                to: ctx.accounts.borrower_ata.to_account_info(),
                authority: liquidity_pool.to_account_info(),
                mint: wanted_mint.to_account_info(),
            },
            &[seeds],
        ),
        borrow_amount,
        wanted_mint.decimals,
    )?;

    // Step 6: Save borrow info
    borrower_account_info.borrower = borrower.key();
    borrower_account_info.borrowed_from_pool = liquidity_pool.key();
    borrower_account_info.total_borrowed = borrower_account_info.total_borrowed.checked_add(borrow_amount).unwrap();
    borrower_account_info.total_collateral = borrower_account_info.total_collateral.checked_add(amount).unwrap();
    borrower_account_info.borrowed_at = Clock::get()?.unix_timestamp;
    borrower_account_info.repaid_amount = 0;
    borrower_account_info.is_closed = false;

    let borrow_duration_value = match borrow_duration {
        0 => BorrowDuration::TenDays,
        1 => BorrowDuration::TwentyDays,
        2 => BorrowDuration::TwentyDays,
        _ => BorrowDuration::TenDays
    };

    borrower_account_info.borrow_duration = borrow_duration_value;

    liquidity_pool.total_liquidity = liquidity_pool.total_liquidity.checked_sub(borrow_amount).unwrap();

    if borrow_mint == liquidity_pool.mint_a {
        liquidity_pool.total_borrowed_a = liquidity_pool.total_borrowed_a
            .checked_add(borrow_amount)
            .ok_or(ErrorCode::MathOverflow)?;
    } else if borrow_mint == liquidity_pool.mint_b {
        liquidity_pool.total_borrowed_b = liquidity_pool.total_borrowed_b
            .checked_add(borrow_amount)
            .ok_or(ErrorCode::MathOverflow)?;
    };

    liquidity_pool.total_borrowed = liquidity_pool
        .total_borrowed_a
        .checked_add(liquidity_pool.total_borrowed_b)
        .ok_or(ErrorCode::MathOverflow)?;


    msg!("Borrower Account Info: {:?}", borrower_account_info);
    msg!("Pool Info: {:?}", liquidity_pool);

    Ok(())
}
