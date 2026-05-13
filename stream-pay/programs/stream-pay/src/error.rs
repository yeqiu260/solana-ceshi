use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Invalid amount - must be greater than zero")]
    InvalidAmount,

    #[msg("Insufficient funds in token account")]
    InsufficientFunds,

    #[msg("Stream has already started")]
    StreamAlreadyStarted,

    #[msg("Stream has not started yet")]
    StreamNotStarted,

    #[msg("Stream has already ended")]
    StreamAlreadyEnded,

    #[msg("Stream has not ended yet")]
    StreamNotEnded,

    #[msg("Unauthorized - caller is not the payer or recipient")]
    Unauthorized,

    #[msg("Arithmetic overflow in calculation")]
    ArithmeticOverflow,

    #[msg("Invalid timestamp - must be greater than current time")]
    InvalidTimestamp,

    #[msg("Invalid rate - must be greater than zero")]
    InvalidRate,

    #[msg("Invalid duration - end time must be after start time")]
    InvalidDuration,

    #[msg("Stream is paused")]
    StreamIsPaused,

    #[msg("Stream is not paused")]
    StreamNotPaused,

    #[msg("Invalid token account for this stream")]
    InvalidTokenAccount,

    #[msg("Nothing to withdraw - no tokens available")]
    NothingToWithdraw,

    #[msg("Amount exceeds remaining stream balance")]
    AmountExceedsBalance,

    #[msg("Rate exceeds maximum allowed")]
    RateTooHigh,

    #[msg("Stream ID does not match")]
    InvalidStreamId,
}
