use anchor_lang::prelude::*;

#[account]
#[derive(Debug, InitSpace)]
pub struct BorrowInfo {
    pub borrower: Pubkey,
    pub borrowed_from_pool: Pubkey,
    pub total_borrowed: u64,
    pub total_collateral: u64,
    pub borrowed_at: i64,
    pub borrow_duration: BorrowDuration,
    pub repaid_amount: u64,
    pub is_closed: bool, // mark when loan is fully repaid
}

#[derive(Clone, Debug, Copy, PartialEq, InitSpace, AnchorSerialize, AnchorDeserialize)]
#[repr(u8)] 
pub enum BorrowDuration {
    TenDays = 10,
    TwentyDays = 20,
    ThirtyDays = 30
}