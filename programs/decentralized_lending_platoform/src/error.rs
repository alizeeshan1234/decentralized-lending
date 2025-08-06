use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Custom error message")]
    CustomError,

    #[msg("Both Token A and Token B mints are same")]
    SameTokenMints,

    InvalidAuthority,

    InvalidLtvThreshold,

    InvalidPenalty,

    InvalidInterestRate,

    InvalidLiquidityAmount,

    Overflow,

    InvalidMint,

    MathOverflow,

    InvalidLtv,

    InvalidLiquidationThreshold,

    InvalidLiquidationPanelty,

    InvalidDuration,

    InvalidRepayAmount,

    LoanNotExpired,

    InvalidBorrower
}
