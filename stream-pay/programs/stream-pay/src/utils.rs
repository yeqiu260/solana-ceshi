use anchor_lang::prelude::*;

use crate::constants::PRECISION;
use crate::error::ErrorCode;

/// Calculate the streamed token amount based on elapsed time.
///
/// Formula: amount = rate * elapsed_seconds / PRECISION
pub fn calculate_streamed_amount(
    rate: u64,
    start_time: i64,
    end_time: i64,
    current_time: i64,
) -> Result<u64> {
    if current_time <= start_time {
        return Ok(0);
    }

    let effective_end = if end_time == 0 || current_time < end_time {
        current_time
    } else {
        end_time
    };

    let elapsed = effective_end
        .checked_sub(start_time)
        .ok_or_else(|| error!(ErrorCode::ArithmeticOverflow))?;

    if elapsed <= 0 {
        return Ok(0);
    }

    let elapsed_u64: u64 = elapsed
        .try_into()
        .map_err(|_| error!(ErrorCode::ArithmeticOverflow))?;

    let amount = rate
        .checked_mul(elapsed_u64)
        .ok_or_else(|| error!(ErrorCode::ArithmeticOverflow))?
        .checked_div(PRECISION)
        .ok_or_else(|| error!(ErrorCode::ArithmeticOverflow))?;

    Ok(amount)
}

/// Calculate the unstreamed (refundable) amount.
pub fn calculate_refund_amount(total_amount: u64, streamed_amount: u64) -> Result<u64> {
    total_amount
        .checked_sub(streamed_amount)
        .ok_or_else(|| error!(ErrorCode::ArithmeticOverflow))
}

/// Multiply a u64 by a ratio (numerator/denominator) safely.
pub fn checked_mul_div(amount: u64, num: u64, den: u64) -> Result<u64> {
    if den == 0 {
        return Err(error!(ErrorCode::ArithmeticOverflow));
    }
    let result = (amount as u128)
        .checked_mul(num as u128)
        .ok_or_else(|| error!(ErrorCode::ArithmeticOverflow))?
        .checked_div(den as u128)
        .ok_or_else(|| error!(ErrorCode::ArithmeticOverflow))?;

    Ok(result as u64)
}

/// Convert a per-second rate to an APR basis-point representation.
pub fn rate_to_apr_bps(rate: u64) -> Result<u16> {
    use crate::constants::{BPS_DENOMINATOR, SECONDS_PER_YEAR};

    let bps = (rate as u128)
        .checked_mul(SECONDS_PER_YEAR as u128)
        .ok_or_else(|| error!(ErrorCode::ArithmeticOverflow))?
        .checked_mul(BPS_DENOMINATOR as u128)
        .ok_or_else(|| error!(ErrorCode::ArithmeticOverflow))?
        .checked_div(PRECISION as u128)
        .ok_or_else(|| error!(ErrorCode::ArithmeticOverflow))?;

    Ok(bps as u16)
}

/// Convert APR basis points to a per-second rate.
pub fn apr_bps_to_rate(apr_bps: u16) -> Result<u64> {
    use crate::constants::{BPS_DENOMINATOR, SECONDS_PER_YEAR};

    let rate = (apr_bps as u128)
        .checked_mul(PRECISION as u128)
        .ok_or_else(|| error!(ErrorCode::ArithmeticOverflow))?
        .checked_div(SECONDS_PER_YEAR as u128 * BPS_DENOMINATOR as u128)
        .ok_or_else(|| error!(ErrorCode::ArithmeticOverflow))?;

    Ok(rate as u64)
}

/// Validate that a timestamp is in a reasonable range (must be in the future).
pub fn validate_timestamp(timestamp: i64, current_time: i64) -> Result<()> {
    if timestamp <= current_time {
        return Err(error!(ErrorCode::InvalidTimestamp));
    }
    Ok(())
}

/// Validate rate is non-zero.
pub fn validate_rate(rate: u64) -> Result<()> {
    if rate == 0 {
        return Err(error!(ErrorCode::InvalidRate));
    }
    Ok(())
}

/// Validate stream duration is within allowed bounds.
pub fn validate_duration(start_time: i64, end_time: i64) -> Result<()> {
    use crate::constants::{MAX_STREAM_DURATION, MIN_STREAM_DURATION};

    let duration = end_time
        .checked_sub(start_time)
        .ok_or_else(|| error!(ErrorCode::InvalidDuration))?;

    if duration < MIN_STREAM_DURATION {
        return Err(error!(ErrorCode::InvalidDuration));
    }

    if duration > MAX_STREAM_DURATION {
        return Err(error!(ErrorCode::InvalidDuration));
    }

    Ok(())
}
