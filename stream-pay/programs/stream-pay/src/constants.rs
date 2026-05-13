use anchor_lang::prelude::*;

/// Precision for per-second rate calculations (9 decimal places)
#[constant]
pub const PRECISION: u64 = 1_000_000_000;

/// Denominator for basis-point calculations
#[constant]
pub const BPS_DENOMINATOR: u16 = 10_000;

/// Maximum fee rate in basis points (100%)
#[constant]
pub const MAX_RATE_BPS: u16 = BPS_DENOMINATOR;

/// PDA seed for stream accounts
#[constant]
pub const STREAM_SEED: &[u8] = b"stream";

/// PDA seed for vault token accounts
#[constant]
pub const VAULT_SEED: &[u8] = b"vault";

/// Seconds in a year (365 days, for APR calculations)
#[constant]
pub const SECONDS_PER_YEAR: u64 = 31_536_000;

/// Minimum duration for a stream (1 second)
#[constant]
pub const MIN_STREAM_DURATION: i64 = 1;

/// Maximum duration for a stream (10 years)
#[constant]
pub const MAX_STREAM_DURATION: i64 = 315_360_000;
